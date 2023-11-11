use core::{future::Future, pin::Pin};

use alloc::{
    boxed::Box,
    collections::{BTreeMap, VecDeque},
    vec::Vec,
};
use jrinx_addr::VirtAddr;
use jrinx_error::{InternalError, Result};
use jrinx_hal::{Cpu, Hal, HaltReason, Interrupt};
use jrinx_percpu::percpu;
use spin::{Mutex, Once};

use crate::{
    arch::{self, SwitchContext},
    executor::{Executor, ExecutorPriority},
    inspector::{Inspector, InspectorId, InspectorMode, InspectorStatus},
    Task, TaskPriority,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RuntimeStatus {
    Init,
    Idle,
    Running(InspectorId),
    Endpoint,
}

pub struct Runtime {
    inspector_registry: BTreeMap<InspectorId, Inspector>,
    inspector_queue: VecDeque<InspectorId>,
    inspector_switch_pending: bool,
    status: RuntimeStatus,
    switch_context: SwitchContext,
}

impl Runtime {
    pub fn new(root_inspector: Inspector) -> Mutex<Pin<Box<Self>>> {
        let mut runtime = Box::pin(Self {
            inspector_registry: BTreeMap::new(),
            inspector_queue: VecDeque::new(),
            inspector_switch_pending: false,
            status: RuntimeStatus::Init,
            switch_context: SwitchContext::new_runtime(),
        });

        runtime.register_inspector(root_inspector).unwrap();

        Mutex::new(runtime)
    }

    pub fn register_inspector(&mut self, inspector: Inspector) -> Result<()> {
        let id = inspector.id();
        self.inspector_registry
            .try_insert(id, inspector)
            .map_err(|_| InternalError::DuplicateInspectorId)?;
        self.inspector_queue.push_back(id);
        Ok(())
    }

    pub fn unregister_inspector(&mut self, id: InspectorId) -> Result<()> {
        self.inspector_registry
            .remove(&id)
            .ok_or(InternalError::InvalidInspectorId)?;
        Ok(())
    }

    pub fn with_current<F, R>(f: F) -> Result<R>
    where
        F: FnOnce(&mut Runtime) -> R,
    {
        hal!().interrupt().with_saved_off(|| {
            RUNTIME.with_ref(|rt| {
                if let Some(rt) = rt.get() {
                    Ok(f(&mut rt.lock()))
                } else {
                    Err(InternalError::InvalidRuntimeStatus)
                }
            })
        })
    }

    pub fn set_inspector_switch_pending(&mut self) {
        self.inspector_switch_pending = true;
    }

    pub fn start() -> ! {
        debug!("runtime started running all inspectors");

        hal!().interrupt().enable();

        let runtime_switch_ctx = Self::with_current(|rt| rt.switch_context_addr()).unwrap();

        loop {
            while let Some(inspector_id) = Self::with_current(|rt| rt.pop_inspector()).unwrap() {
                trace!("switch into inspector {:?}", inspector_id);

                Self::with_current(|rt| rt.set_current_inspector(Some(inspector_id))).unwrap();

                Inspector::run(runtime_switch_ctx);

                Self::with_current(|rt| {
                    rt.clr_inspector_switch_pending();
                    rt.set_current_inspector(None);
                })
                .unwrap();

                trace!("switch from inspector {:?}", inspector_id);

                if Self::with_current(|rt| {
                    rt.with_inspector(inspector_id, |is| is.status() == InspectorStatus::Finished)
                        .unwrap()
                })
                .unwrap()
                {
                    Self::with_current(|rt| rt.unregister_inspector(inspector_id).unwrap())
                        .unwrap();
                } else {
                    Self::with_current(|rt| rt.push_inspector(inspector_id).unwrap()).unwrap();
                }
            }

            debug!("runtime finished running all inspectors");

            Self::halt_if_all_finished_or_broadcast_ipi();

            debug!("runtime send ipi and wait");
            hal!().interrupt().wait();
        }
    }

    pub fn switch_yield() {
        let runtime_switch_ctx = Self::with_current(|rt| rt.switch_context_addr()).unwrap();
        let executor_switch_ctx = Executor::with_current(|ex| ex.switch_context()).unwrap();
        unsafe {
            arch::switch(
                executor_switch_ctx.as_usize(),
                runtime_switch_ctx.as_usize(),
            );
        }
    }

    pub(crate) fn status(&self) -> RuntimeStatus {
        self.status
    }

    pub(crate) fn get_inspector_switch_pending(&self) -> bool {
        self.inspector_switch_pending
    }

    pub(crate) fn with_inspector<F, R>(&mut self, id: InspectorId, f: F) -> Result<R>
    where
        F: FnOnce(&mut Inspector) -> R,
    {
        let inspector = self
            .inspector_registry
            .get_mut(&id)
            .ok_or(InternalError::InvalidInspectorId)?;
        Ok(f(inspector))
    }

    fn clr_inspector_switch_pending(&mut self) {
        self.inspector_switch_pending = false;
    }
    fn set_current_inspector(&mut self, id: Option<InspectorId>) {
        if let Some(id) = id {
            match self.status {
                RuntimeStatus::Running(ref mut inspector_id) => {
                    *inspector_id = id;
                }
                _ => {
                    self.status = RuntimeStatus::Running(id);
                }
            }
        } else {
            self.status = RuntimeStatus::Idle;
        }
    }

    fn pop_inspector(&mut self) -> Option<InspectorId> {
        self.inspector_queue.pop_front()
    }

    fn push_inspector(&mut self, id: InspectorId) -> Result<()> {
        if !self.inspector_registry.contains_key(&id) {
            return Err(InternalError::InvalidInspectorId);
        }
        self.inspector_queue.push_back(id);
        Ok(())
    }

    fn switch_context_addr(&self) -> VirtAddr {
        VirtAddr::new(&self.switch_context as *const _ as usize)
    }

    fn halt_if_all_finished_or_broadcast_ipi() {
        while hal!().interrupt().with_saved_off(critical_inner).is_none() {}

        fn critical_inner() -> Option<()> {
            let guards = RUNTIME
                .iter()
                .zip(0usize..)
                .filter_map(|(rt, id)| rt.get().map(|rt| (rt.try_lock(), id)))
                .collect::<Vec<_>>();
            if guards.iter().any(|(guard, _)| guard.is_none()) {
                None
            } else {
                if guards
                    .iter()
                    .all(|(guard, _)| guard.as_ref().unwrap().status == RuntimeStatus::Endpoint)
                {
                    hal!().halt(HaltReason::NormalExit);
                } else {
                    let ipi_target = guards
                        .iter()
                        .filter_map(|(guard, id)| {
                            (guard.as_ref().unwrap().status == RuntimeStatus::Endpoint)
                                .then_some(*id)
                        })
                        .min();

                    guards
                        .into_iter()
                        .find_map(|(guard, id)| (id == hal!().cpu().id()).then_some(guard))
                        .unwrap()
                        .unwrap()
                        .status = RuntimeStatus::Endpoint;

                    if let Some(target) = ipi_target {
                        hal!().interrupt().send_ipi(&[target]);
                    }
                }
                Some(())
            }
        }
    }
}

unsafe impl Send for Runtime {}

#[percpu]
static RUNTIME: Once<Mutex<Pin<Box<Runtime>>>> = Once::new();

pub fn init(future: impl Future<Output = ()> + 'static) {
    RUNTIME
        .as_ref()
        .try_call_once::<_, ()>(|| {
            Ok(Runtime::new(Inspector::new(
                InspectorMode::Bootstrap,
                Executor::new(
                    ExecutorPriority::default(),
                    Task::new(future, TaskPriority::default()),
                ),
            )))
        })
        .unwrap();
}
