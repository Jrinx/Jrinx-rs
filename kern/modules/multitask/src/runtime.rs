use core::{future::Future, pin::Pin};

use alloc::{
    boxed::Box,
    collections::{BTreeMap, VecDeque},
};
use jrinx_addr::VirtAddr;
use jrinx_error::{InternalError, Result};
use jrinx_hal::{Hal, HaltReason, Interrupt};
use jrinx_percpu::percpu;
use spin::{Mutex, Once};

use crate::{
    arch::{self, SwitchContext},
    inspector::{self, Inspector, InspectorId, InspectorMode, InspectorStatus},
};

use super::{
    executor::{self, Executor, ExecutorPriority},
    Task, TaskPriority,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RuntimeStatus {
    Init,
    Idle,
    Running(InspectorId),
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

    pub fn with_current_inspector<F, R>(&mut self, f: F) -> Result<R>
    where
        F: FnOnce(&mut Inspector) -> R,
    {
        let RuntimeStatus::Running(inspector_id) = self.status else {
            return Err(InternalError::InvalidRuntimeStatus);
        };
        self.with_inspector(inspector_id, f)
    }

    pub(crate) fn get_inspector_switch_pending(&self) -> bool {
        self.inspector_switch_pending
    }

    pub fn set_inspector_switch_pending(&mut self) {
        self.inspector_switch_pending = true;
    }

    fn clr_inspector_switch_pending(&mut self) {
        self.inspector_switch_pending = false;
    }

    fn with_inspector<F, R>(&mut self, id: InspectorId, f: F) -> Result<R>
    where
        F: FnOnce(&mut Inspector) -> R,
    {
        let inspector = self
            .inspector_registry
            .get_mut(&id)
            .ok_or(InternalError::InvalidInspectorId)?;
        Ok(f(inspector))
    }

    fn set_current_inspector(&mut self, id: Option<InspectorId>) {
        if let Some(id) = id {
            match self.status {
                RuntimeStatus::Running(ref mut inspector_id) => {
                    *inspector_id = id;
                }
                RuntimeStatus::Init | RuntimeStatus::Idle => {
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
}

unsafe impl Send for Runtime {}

#[percpu]
static RUNTIME: Once<Mutex<Pin<Box<Runtime>>>> = Once::new();

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

pub fn start() -> ! {
    debug!("runtime started running all inspectors");

    hal!().interrupt().enable();

    let runtime_switch_ctx = with_current(|rt| rt.switch_context_addr()).unwrap();
    while let Some(inspector_id) = with_current(|rt| rt.pop_inspector()).unwrap() {
        debug!("runtime running inspector {:?}", inspector_id);

        with_current(|rt| rt.set_current_inspector(Some(inspector_id))).unwrap();

        inspector::run(runtime_switch_ctx);

        with_current(|rt| {
            rt.clr_inspector_switch_pending();
            rt.set_current_inspector(None);
        })
        .unwrap();

        if with_current(|rt| {
            rt.with_inspector(inspector_id, |is| is.status() == InspectorStatus::Finished)
                .unwrap()
        })
        .unwrap()
        {
            with_current(|rt| rt.unregister_inspector(inspector_id).unwrap()).unwrap();
        } else {
            with_current(|rt| rt.push_inspector(inspector_id).unwrap()).unwrap();
        }
    }

    debug!("runtime finished running all inspectors");

    hal!().halt(HaltReason::NormalExit);
}

pub fn switch_yield() {
    let runtime_switch_ctx = with_current(|rt| rt.switch_context_addr()).unwrap();
    let executor_switch_ctx = executor::with_current(|ex| ex.switch_context_addr()).unwrap();
    unsafe {
        arch::switch(
            executor_switch_ctx.as_usize(),
            runtime_switch_ctx.as_usize(),
        );
    }
}
