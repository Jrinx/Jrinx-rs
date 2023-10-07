use core::pin::Pin;

use alloc::{boxed::Box, collections::BTreeMap};

use crate::{
    arch::{self, task::executor::SwitchContext},
    cpudata,
    error::{HaltReason, InternalError, Result},
    mm::virt::VirtAddr,
    util::priority::PriorityQueue,
};

use super::executor::{
    Executor, ExecutorBehaviorOnNoTask, ExecutorId, ExecutorPriority, ExecutorStatus,
};

type ExecutorQueue = PriorityQueue<ExecutorPriority, ExecutorId>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RuntimeStatus {
    Init,
    Idle,
    Running(ExecutorId),
}

pub struct Runtime {
    executor_registry: BTreeMap<ExecutorId, Pin<Box<Executor>>>,
    executor_queue: ExecutorQueue,
    status: RuntimeStatus,
    switch_context: SwitchContext,
}

impl Runtime {
    pub fn new() -> Pin<Box<Self>> {
        let mut runtime = Box::pin(Self {
            executor_registry: BTreeMap::new(),
            executor_queue: ExecutorQueue::new(),
            status: RuntimeStatus::Init,
            switch_context: SwitchContext::new_runtime(),
        });

        runtime
            .register_executor(Executor::new(
                ExecutorPriority::new(0),
                ExecutorBehaviorOnNoTask::EXIT,
            ))
            .unwrap();

        runtime
    }

    pub fn register_executor(&mut self, executor: Pin<Box<Executor>>) -> Result<()> {
        let id = executor.id();
        let priority = executor.priority();
        self.executor_registry
            .try_insert(id, executor)
            .map_err(|_| InternalError::DuplicateExecutorId)?;
        self.executor_queue.add(id, priority);
        Ok(())
    }

    pub fn unregister_executor(&mut self, id: ExecutorId) -> Result<()> {
        if self.executor_registry.len() < 1 {
            return Err(InternalError::InvalidRuntimeStatus);
        }
        self.executor_registry
            .remove(&id)
            .ok_or(InternalError::InvalidExecutorId)?;
        Ok(())
    }

    pub fn with_current_executor<F, R>(&mut self, f: F) -> Result<R>
    where
        F: FnOnce(&mut Pin<Box<Executor>>) -> R,
    {
        let RuntimeStatus::Running(executor_id) = self.status else {
            return Err(InternalError::InvalidRuntimeStatus);
        };
        let executor = self
            .executor_registry
            .get_mut(&executor_id)
            .ok_or(InternalError::InvalidRuntimeStatus)?;
        Ok(f(executor))
    }

    pub fn with_specific_executor<F, R>(&mut self, id: ExecutorId, f: F) -> Result<R>
    where
        F: FnOnce(&mut Pin<Box<Executor>>) -> R,
    {
        let executor = self
            .executor_registry
            .get_mut(&id)
            .ok_or(InternalError::InvalidExecutorId)?;
        Ok(f(executor))
    }

    pub fn with_bootstrap_executor<F, R>(&mut self, f: F) -> Result<R>
    where
        F: FnOnce(&mut Pin<Box<Executor>>) -> R,
    {
        let RuntimeStatus::Init = self.status else {
            return Err(InternalError::InvalidRuntimeStatus);
        };
        if self.executor_registry.len() < 1 {
            return Err(InternalError::InvalidRuntimeStatus);
        }
        let executor_id = self.executor_queue.pop().unwrap();
        let executor = self.executor_registry.get_mut(&executor_id).unwrap();
        let result = f(executor);
        self.executor_queue.add(executor_id, executor.priority());
        Ok(result)
    }

    fn set_current_executor(&mut self, id: Option<ExecutorId>) {
        if let Some(id) = id {
            match self.status {
                RuntimeStatus::Running(ref mut executor_id) => {
                    *executor_id = id;
                }
                RuntimeStatus::Init | RuntimeStatus::Idle => {
                    self.status = RuntimeStatus::Running(id);
                }
            }
        } else {
            self.status = RuntimeStatus::Idle;
        }
    }

    fn pop_executor(&mut self) -> Option<ExecutorId> {
        self.executor_queue.pop()
    }

    fn add_executor(&mut self, id: ExecutorId) -> Result<()> {
        let Some(executor) = self.executor_registry.get_mut(&id) else {
            return Err(InternalError::InvalidExecutorId);
        };
        self.executor_queue.add(id, executor.priority());
        Ok(())
    }

    fn executor_switch_context_addr(&mut self, id: ExecutorId) -> Option<VirtAddr> {
        self.executor_registry
            .get_mut(&id)
            .map(|executor| executor.switch_context_addr())
    }

    fn switch_context_addr(&mut self) -> VirtAddr {
        VirtAddr::new(&mut self.switch_context as *mut _ as usize)
    }
}

pub fn start() -> ! {
    let runtime_switch_ctx = cpudata::with_cpu_runtime(|rt| rt.switch_context_addr());
    while let Some(executor_id) = cpudata::with_cpu_runtime(|rt| rt.pop_executor()) {
        cpudata::with_cpu_runtime(|rt| rt.set_current_executor(Some(executor_id)));

        let executor_switch_ctx =
            cpudata::with_cpu_runtime(|rt| rt.executor_switch_context_addr(executor_id)).unwrap();
        // TODO: setup timer
        unsafe {
            arch::task::executor::switch(
                runtime_switch_ctx.as_usize(),
                executor_switch_ctx.as_usize(),
            );
        }

        cpudata::with_cpu_runtime(|rt| rt.set_current_executor(None));

        if cpudata::with_cpu_runtime(|rt| {
            rt.with_specific_executor(executor_id, |executor| {
                executor.status() == ExecutorStatus::Finished
            })
            .unwrap()
        }) {
            cpudata::with_cpu_runtime(|rt| rt.unregister_executor(executor_id).unwrap());
        } else {
            cpudata::with_cpu_runtime(|rt| rt.add_executor(executor_id).unwrap());
        }
    }
    info!("all executors finished");
    arch::halt(HaltReason::NormalExit);
}

pub fn switch_yield() {
    let runtime_switch_ctx = cpudata::with_cpu_runtime(|rt| rt.switch_context_addr());
    let executor_switch_ctx = cpudata::with_cpu_executor(|ex| ex.switch_context_addr());
    unsafe {
        arch::task::executor::switch(
            executor_switch_ctx.as_usize(),
            runtime_switch_ctx.as_usize(),
        );
    }
}
