pub mod inspector;

use core::pin::Pin;

use alloc::{
    boxed::Box,
    collections::{BTreeMap, VecDeque},
};
use jrinx_addr::VirtAddr;
use jrinx_error::{InternalError, Result};
use jrinx_hal::{Hal, HaltReason, Interrupt};

use crate::{
    arch::{self, task::executor::SwitchContext},
    cpudata::CpuDataVisitor,
    task::runtime::inspector::InspectorStatus,
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
    debug!("runtime started running all inspectors");

    hal!().interrupt().enable();

    let runtime_switch_ctx = CpuDataVisitor::new()
        .runtime(|rt| rt.switch_context_addr())
        .unwrap();
    while let Some(inspector_id) = CpuDataVisitor::new()
        .runtime(|rt| rt.pop_inspector())
        .unwrap()
    {
        debug!("runtime running inspector {:?}", inspector_id);

        CpuDataVisitor::new()
            .runtime(|rt| rt.set_current_inspector(Some(inspector_id)))
            .unwrap();

        inspector::run(runtime_switch_ctx);

        CpuDataVisitor::new()
            .runtime(|rt| rt.clr_inspector_switch_pending())
            .unwrap();

        CpuDataVisitor::new()
            .runtime(|rt| rt.set_current_inspector(None))
            .unwrap();
        if CpuDataVisitor::new()
            .runtime(|rt| {
                rt.with_inspector(inspector_id, |inspector| {
                    inspector.status() == InspectorStatus::Finished
                })
                .unwrap()
            })
            .unwrap()
        {
            CpuDataVisitor::new()
                .runtime(|rt| rt.unregister_inspector(inspector_id).unwrap())
                .unwrap();
        } else {
            CpuDataVisitor::new()
                .runtime(|rt| rt.push_inspector(inspector_id).unwrap())
                .unwrap();
        }
    }

    debug!("runtime finished running all inspectors");

    hal!().halt(HaltReason::NormalExit);
}

pub fn switch_yield() {
    let runtime_switch_ctx = CpuDataVisitor::new()
        .runtime(|rt| rt.switch_context_addr())
        .unwrap();
    let executor_switch_ctx = CpuDataVisitor::new()
        .executor(|ex| ex.switch_context_addr())
        .unwrap();
    unsafe {
        arch::task::executor::switch(
            executor_switch_ctx.as_usize(),
            runtime_switch_ctx.as_usize(),
        );
    }
}
