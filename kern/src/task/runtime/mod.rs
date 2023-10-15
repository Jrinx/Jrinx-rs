pub mod inspector;

use core::pin::Pin;

use alloc::{
    boxed::Box,
    collections::{BTreeMap, VecDeque},
};

use crate::{
    arch::{self, task::executor::SwitchContext},
    cpudata,
    error::{HaltReason, InternalError, Result},
    mm::virt::VirtAddr,
    task::{
        executor::ExecutorStatus,
        runtime::inspector::{InspectorMode, InspectorStatus},
    },
};

use self::inspector::{Inspector, InspectorId};

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
    pub fn new(root_inspector: Inspector) -> Pin<Box<Self>> {
        let mut runtime = Box::pin(Self {
            inspector_registry: BTreeMap::new(),
            inspector_queue: VecDeque::new(),
            inspector_switch_pending: false,
            status: RuntimeStatus::Init,
            switch_context: SwitchContext::new_runtime(),
        });

        runtime.register_inspector(root_inspector).unwrap();

        runtime
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

    fn switch_context_addr(&mut self) -> VirtAddr {
        VirtAddr::new(&mut self.switch_context as *mut _ as usize)
    }
}

pub fn start() -> ! {
    info!("runtime started running all inspectors");

    arch::int_enable();

    let runtime_switch_ctx = cpudata::with_cpu_runtime(|rt| rt.switch_context_addr()).unwrap();
    while let Some(inspector_id) = cpudata::with_cpu_runtime(|rt| rt.pop_inspector()).unwrap() {
        debug!("runtime running inspector {:?}", inspector_id);

        cpudata::with_cpu_runtime(|rt| rt.set_current_inspector(Some(inspector_id))).unwrap();

        loop {
            if cpudata::with_cpu_runtime(|rt| rt.inspector_switch_pending).unwrap()
                || cpudata::with_cpu_inspector(|inspector| {
                    inspector.status() == InspectorStatus::Finished
                })
                .unwrap()
            {
                break;
            }

            let Some(executor_id) =
                cpudata::with_cpu_inspector(|inspector| inspector.pop_executor()).unwrap()
            else {
                arch::wait_for_interrupt();
                continue;
            };
            trace!("switch to executor {:?}", executor_id);

            cpudata::with_cpu_inspector(|inspector| {
                inspector.set_current_executor(Some(executor_id));
            })
            .unwrap();

            let executor_switch_ctx =
                cpudata::with_cpu_executor(|executor| executor.switch_context_addr()).unwrap();

            unsafe {
                arch::task::executor::switch(
                    runtime_switch_ctx.as_usize(),
                    executor_switch_ctx.as_usize(),
                );
            }
            cpudata::with_cpu_inspector(|inspector| inspector.set_current_executor(None)).unwrap();

            trace!("switch back from executor {:?}", executor_id);

            if cpudata::with_cpu_inspector(|inspector| {
                inspector
                    .with_executor(executor_id, |executor| {
                        executor.status() == ExecutorStatus::Finished
                    })
                    .unwrap()
            })
            .unwrap()
            {
                cpudata::with_cpu_inspector(|inspector| {
                    inspector.unregister_executor(executor_id).unwrap();
                })
                .unwrap();
            } else {
                cpudata::with_cpu_inspector(|inspector| {
                    inspector.push_executor(executor_id).unwrap();
                })
                .unwrap();
            }

            if cpudata::with_cpu_inspector(|inspector| {
                inspector.is_empty() && inspector.mode() == InspectorMode::Bootstrap
            })
            .unwrap()
            {
                cpudata::with_cpu_inspector(|inspector| inspector.mark_finished()).unwrap();
            }
        }

        cpudata::with_cpu_runtime(|rt| rt.clr_inspector_switch_pending()).unwrap();

        cpudata::with_cpu_runtime(|rt| rt.set_current_inspector(None)).unwrap();
        if cpudata::with_cpu_runtime(|rt| {
            rt.with_inspector(inspector_id, |inspector| {
                inspector.status() == InspectorStatus::Finished
            })
            .unwrap()
        })
        .unwrap()
        {
            cpudata::with_cpu_runtime(|rt| rt.unregister_inspector(inspector_id).unwrap()).unwrap();
        } else {
            cpudata::with_cpu_runtime(|rt| rt.push_inspector(inspector_id).unwrap()).unwrap();
        }
    }

    info!("runtime finished running all inspectors");

    arch::halt(HaltReason::NormalExit);
}

pub fn switch_yield() {
    let runtime_switch_ctx = cpudata::with_cpu_runtime(|rt| rt.switch_context_addr()).unwrap();
    let executor_switch_ctx = cpudata::with_cpu_executor(|ex| ex.switch_context_addr()).unwrap();
    unsafe {
        arch::task::executor::switch(
            executor_switch_ctx.as_usize(),
            runtime_switch_ctx.as_usize(),
        );
    }
}
