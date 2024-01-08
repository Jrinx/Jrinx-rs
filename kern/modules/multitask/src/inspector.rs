use core::{fmt::Display, pin::Pin};

use alloc::{boxed::Box, collections::BTreeMap};
use jrinx_addr::VirtAddr;
use jrinx_error::{InternalError, Result};
use jrinx_serial_id_macro::SerialId;
use jrinx_util::fastpq::FastPriorityQueueWithLock;
use spin::{Mutex, RwLock};

use crate::{
    arch,
    executor::{Executor, ExecutorId, ExecutorPriority, ExecutorStatus},
    runtime::{Runtime, RuntimeStatus},
};

type ExecutorQueue = FastPriorityQueueWithLock<ExecutorPriority, ExecutorId>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, SerialId)]
pub struct InspectorId(u64);

impl Display for InspectorId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InspectorStatus {
    Idle,
    Running(ExecutorId),
    Pending(ExecutorId),
}

pub struct Inspector {
    id: InspectorId,
    status: Mutex<InspectorStatus>,
    scheduler: RwLock<Scheduler>,
}

struct Scheduler {
    registry: BTreeMap<ExecutorId, Pin<Box<Executor>>>,
    queue: ExecutorQueue,
}

impl Inspector {
    pub fn new(root_executor: Pin<Box<Executor>>) -> Self {
        let inspector = Self {
            id: InspectorId::new(),
            status: Mutex::new(InspectorStatus::Idle),
            scheduler: RwLock::new(Scheduler {
                registry: BTreeMap::new(),
                queue: ExecutorQueue::new(),
            }),
        };

        inspector.register(root_executor).unwrap();

        inspector
    }

    pub fn id(&self) -> InspectorId {
        self.id
    }

    pub fn status(&self) -> InspectorStatus {
        *self.status.lock()
    }

    pub fn is_empty(&self) -> bool {
        self.scheduler.read().registry.is_empty()
    }

    pub fn mark_pending(&self) -> Result<()> {
        let mut status = self.status.lock();

        if let InspectorStatus::Idle = *status {
            return Err(InternalError::InvalidInspectorStatus);
        }

        if let InspectorStatus::Running(executor_id) = *status {
            *status = InspectorStatus::Pending(executor_id);
        }
        Ok(())
    }

    pub fn register(&self, executor: Pin<Box<Executor>>) -> Result<()> {
        let mut scheduler = self.scheduler.write();

        let id = executor.id();
        let priority = executor.priority();

        scheduler
            .registry
            .try_insert(id, executor)
            .map_err(|_| InternalError::DuplicateExecutorId)?;
        scheduler.queue.enqueue(priority, id);
        Ok(())
    }

    pub fn unregister(&self, executor_id: ExecutorId) -> Result<()> {
        self.scheduler
            .write()
            .registry
            .remove(&executor_id)
            .ok_or(InternalError::InvalidExecutorId)?;
        Ok(())
    }

    pub fn with_current<F, R>(f: F) -> Result<R>
    where
        F: FnOnce(&Inspector) -> R,
    {
        Runtime::with_current(|rt| {
            let f = f;
            let f = |is: &_| f(is);
            let RuntimeStatus::Running(inspector_id) = rt.status() else {
                return Err(InternalError::InvalidRuntimeStatus);
            };
            rt.with_inspector(inspector_id, f)
        })
    }

    pub(crate) fn with_executor<F, R>(&self, id: ExecutorId, f: F) -> Result<R>
    where
        F: FnOnce(&mut Pin<Box<Executor>>) -> R,
    {
        Ok(f(self
            .scheduler
            .write()
            .registry
            .get_mut(&id)
            .ok_or(InternalError::InvalidExecutorId)?))
    }

    pub(crate) fn dequeue(&self) -> Option<ExecutorId> {
        self.scheduler.write().queue.dequeue().map(|(_, id)| id)
    }

    pub(crate) fn enqueue(&self, id: ExecutorId) -> Result<()> {
        let scheduler = self.scheduler.write();

        let Some(executor) = scheduler.registry.get(&id) else {
            return Err(InternalError::InvalidExecutorId);
        };
        scheduler.queue.enqueue(executor.priority(), id);
        Ok(())
    }

    pub(crate) fn set_current(&self, id: Option<ExecutorId>) {
        let mut status = self.status.lock();

        if let Some(id) = id {
            match *status {
                InspectorStatus::Running(ref mut executor_id) => {
                    *executor_id = id;
                }
                _ => {
                    *status = InspectorStatus::Running(id);
                }
            }
        } else if let InspectorStatus::Running(_) = *status {
            *status = InspectorStatus::Idle;
        }
    }

    pub(crate) fn run(runtime_switch_ctx: VirtAddr) {
        loop {
            let Some(executor_id) = Inspector::with_current(|is| is.dequeue()).unwrap() else {
                break;
            };
            trace!("switch into executor {:?}", executor_id);

            Inspector::with_current(|is| {
                is.set_current(Some(executor_id));
            })
            .unwrap();

            let executor_switch_ctx = Executor::with_current(|ex| ex.switch_context()).unwrap();

            unsafe {
                arch::switch(
                    runtime_switch_ctx.as_usize(),
                    executor_switch_ctx.as_usize(),
                );
            }

            Inspector::with_current(|is| is.set_current(None)).unwrap();

            trace!("switch from executor {:?}", executor_id);

            let switch_out = Inspector::with_current(|is| {
                if is
                    .with_executor(executor_id, |ex| ex.status() == ExecutorStatus::Finished)
                    .unwrap()
                {
                    is.unregister(executor_id).unwrap();
                } else {
                    is.enqueue(executor_id).unwrap();
                }
                matches!(is.status(), InspectorStatus::Pending(_))
            })
            .unwrap();

            if switch_out {
                break;
            }
        }
    }
}
