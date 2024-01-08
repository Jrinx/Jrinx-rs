use core::{
    fmt::Display,
    pin::Pin,
    task::{Context, Poll, Waker},
};

use alloc::{boxed::Box, collections::BTreeMap, sync::Arc, task::Wake};
use jrinx_addr::VirtAddr;
use jrinx_error::{InternalError, Result};
use jrinx_hal::{Hal, Vm};
use jrinx_paging::{GenericPagePerm, GenericPageTable, PagePerm};
use jrinx_phys_frame::PhysFrame;
use jrinx_serial_id_macro::SerialId;
use jrinx_stack_alloc::StackAllocator;
use jrinx_util::fastpq::{FastPriority, FastPriorityQueueWithLock};
use jrinx_vmm::KERN_PAGE_TABLE;
use spin::Lazy;

use crate::{
    arch::{self, SwitchContext},
    inspector::{Inspector, InspectorStatus},
    runtime::Runtime,
    Task, TaskId, TaskPriority,
};

type TaskQueue = FastPriorityQueueWithLock<TaskPriority, TaskId>;

static EXECUTOR_STACK_ALLOCATOR: Lazy<StackAllocator> = Lazy::new(|| {
    StackAllocator::new(
        (
            VirtAddr::new(jrinx_config::EXECUTOR_STACK_REGION.addr),
            jrinx_config::EXECUTOR_STACK_REGION.len,
        ),
        jrinx_config::EXECUTOR_STACK_SIZE,
        |addr| {
            let mut page_table = KERN_PAGE_TABLE.write();
            let phys_frame = PhysFrame::alloc()?;
            page_table.map(addr, phys_frame, PagePerm::G | PagePerm::R | PagePerm::W)?;
            Ok(())
        },
        |addr| {
            let mut page_table = KERN_PAGE_TABLE.write();
            page_table.unmap(addr)?;
            Ok(())
        },
    )
});

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, SerialId)]
pub struct ExecutorId(u64);

impl Display for ExecutorId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ExecutorPriority(FastPriority);

impl ExecutorPriority {
    pub const fn new(priority: u8) -> Self {
        Self(FastPriority::new(priority))
    }
}

impl From<ExecutorPriority> for FastPriority {
    fn from(value: ExecutorPriority) -> Self {
        value.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutorStatus {
    Runnable,
    Finished,
}

pub struct Executor {
    id: ExecutorId,
    priority: ExecutorPriority,
    status: ExecutorStatus,
    stack_top: VirtAddr,
    switch_context: SwitchContext,
    task_registry: BTreeMap<TaskId, Task>,
    task_queue: Arc<TaskQueue>,
    task_waker: BTreeMap<TaskId, Waker>,
}

impl Executor {
    pub fn new(priority: ExecutorPriority, root_task: Task) -> Pin<Box<Self>> {
        let entry = VirtAddr::new(arch::executor_launch as usize);
        let stack_top = EXECUTOR_STACK_ALLOCATOR
            .allocate(jrinx_config::EXECUTOR_STACK_SIZE)
            .unwrap();

        hal!().vm().sync_all();

        let mut executor = Box::pin(Self {
            id: ExecutorId::new(),
            priority,
            status: ExecutorStatus::Runnable,
            stack_top,
            switch_context: SwitchContext::new_executor(entry, stack_top),
            task_registry: BTreeMap::new(),
            task_queue: Arc::new(TaskQueue::new()),
            task_waker: BTreeMap::new(),
        });

        let executor_addr = &*executor as *const _ as usize;

        executor
            .switch_context
            .init_executor_addr(VirtAddr::new(executor_addr));

        executor.spawn(root_task).unwrap();

        executor
    }

    pub fn id(&self) -> ExecutorId {
        self.id
    }

    pub fn priority(&self) -> ExecutorPriority {
        self.priority
    }

    pub fn status(&self) -> ExecutorStatus {
        self.status
    }

    pub fn spawn(&mut self, task: Task) -> Result<&mut Self> {
        let id = task.id;
        self.task_queue.enqueue(task.priority, id);
        self.task_registry
            .try_insert(id, task)
            .map_err(|_| InternalError::DuplicateTaskId)?;
        Ok(self)
    }

    pub fn with_current<F, R>(f: F) -> Result<R>
    where
        F: FnOnce(&mut Pin<Box<Executor>>) -> R,
    {
        Inspector::with_current(|is| {
            let executor_id = match is.status() {
                InspectorStatus::Running(executor_id) | InspectorStatus::Pending(executor_id) => {
                    executor_id
                }
                _ => return Err(InternalError::InvalidInspectorStatus),
            };
            is.with_executor(executor_id, f)
        })?
    }

    pub(crate) fn switch_context(&self) -> VirtAddr {
        VirtAddr::new(&self.switch_context as *const _ as usize)
    }

    pub(crate) fn run(&mut self) {
        let Self {
            task_registry,
            task_queue,
            task_waker,
            ..
        } = self;

        while let Some((_, task_id)) = task_queue.dequeue() {
            let task = match task_registry.get_mut(&task_id) {
                Some(task) => task,
                None => continue,
            };

            let waker = task_waker
                .entry(task_id)
                .or_insert_with(|| TaskWaker::create(task.id, task.priority, task_queue.clone()));

            let mut context = Context::from_waker(waker);
            match task.poll(&mut context) {
                Poll::Ready(()) => {
                    task_registry.remove(&task_id);
                    task_waker.remove(&task_id);
                }
                Poll::Pending => {}
            }
        }

        self.status = ExecutorStatus::Finished;
    }

    pub(crate) fn start(address: usize) -> ! {
        let mut executor = unsafe { Box::from_raw(address as *mut Executor) };
        executor.run();

        Runtime::switch_yield();
        unreachable!();
    }
}

impl Drop for Executor {
    fn drop(&mut self) {
        EXECUTOR_STACK_ALLOCATOR.deallocate(self.stack_top).unwrap();

        hal!().vm().sync_all();
    }
}

struct TaskWaker {
    task_id: TaskId,
    task_priority: TaskPriority,
    task_queue: Arc<TaskQueue>,
}

impl Wake for TaskWaker {
    fn wake(self: Arc<Self>) {
        self.wake_task();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.wake_task();
    }
}

impl TaskWaker {
    fn create(task_id: TaskId, task_priority: TaskPriority, task_queue: Arc<TaskQueue>) -> Waker {
        Waker::from(Arc::new(Self {
            task_id,
            task_priority,
            task_queue,
        }))
    }

    fn wake_task(&self) {
        self.task_queue.enqueue(self.task_priority, self.task_id);
    }
}
