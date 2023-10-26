use core::{fmt::Display, pin::Pin};

use alloc::{boxed::Box, collections::BTreeMap};
use jrinx_error::{InternalError, Result};
use jrinx_serial_id::SerialIdGenerator;
use jrinx_serial_id_macro::SerialId;

use crate::{
    task::executor::{Executor, ExecutorId, ExecutorPriority},
    util::priority::PriorityQueueWithLock,
};

type ExecutorQueue = PriorityQueueWithLock<ExecutorPriority, ExecutorId>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, SerialId)]
pub struct InspectorId(u64);

impl Display for InspectorId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InspectorMode {
    Bootstrap,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InspectorStatus {
    Idle,
    Running(ExecutorId),
    Finished,
}

pub struct Inspector {
    id: InspectorId,
    mode: InspectorMode,
    status: InspectorStatus,
    executor_registry: BTreeMap<ExecutorId, Pin<Box<Executor>>>,
    executor_queue: ExecutorQueue,
}

impl Inspector {
    pub fn new(mode: InspectorMode, root_executor: Pin<Box<Executor>>) -> Self {
        let mut inspector = Self {
            id: InspectorId::generate(),
            mode,
            status: InspectorStatus::Idle,
            executor_registry: BTreeMap::new(),
            executor_queue: ExecutorQueue::new(),
        };

        inspector.register_executor(root_executor).unwrap();

        inspector
    }

    pub fn id(&self) -> InspectorId {
        self.id
    }

    pub fn mode(&self) -> InspectorMode {
        self.mode
    }

    pub fn status(&self) -> InspectorStatus {
        self.status
    }

    pub fn is_empty(&self) -> bool {
        self.executor_registry.is_empty()
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

    pub fn unregister_executor(&mut self, executor_id: ExecutorId) -> Result<()> {
        self.executor_registry
            .remove(&executor_id)
            .ok_or(InternalError::InvalidExecutorId)?;
        Ok(())
    }

    pub fn with_current_executor<F, R>(&mut self, f: F) -> Result<R>
    where
        F: FnOnce(&mut Pin<Box<Executor>>) -> R,
    {
        let InspectorStatus::Running(executor_id) = self.status else {
            return Err(InternalError::InvalidInspectorStatus);
        };
        self.with_executor(executor_id, f)
    }

    pub fn mark_finished(&mut self) {
        self.status = InspectorStatus::Finished;
    }

    pub(super) fn with_executor<F, R>(&mut self, id: ExecutorId, f: F) -> Result<R>
    where
        F: FnOnce(&mut Pin<Box<Executor>>) -> R,
    {
        let executor = self
            .executor_registry
            .get_mut(&id)
            .ok_or(InternalError::InvalidExecutorId)?;
        Ok(f(executor))
    }

    pub(super) fn pop_executor(&mut self) -> Option<ExecutorId> {
        self.executor_queue.pop()
    }

    pub(super) fn push_executor(&mut self, id: ExecutorId) -> Result<()> {
        let Some(executor) = self.executor_registry.get(&id) else {
            return Err(InternalError::InvalidExecutorId);
        };
        self.executor_queue.add(id, executor.priority());
        Ok(())
    }

    pub(super) fn set_current_executor(&mut self, id: Option<ExecutorId>) {
        if let Some(id) = id {
            match self.status {
                InspectorStatus::Running(ref mut executor_id) => {
                    *executor_id = id;
                }
                _ => {
                    self.status = InspectorStatus::Running(id);
                }
            }
        } else {
            self.status = InspectorStatus::Idle;
        }
    }
}
