use core::{
    fmt::Display,
    pin::Pin,
    task::{Context, Poll, Waker},
};

use alloc::{boxed::Box, collections::BTreeMap, sync::Arc, task::Wake};
use spin::Mutex;

use crate::{
    arch::{self, mm::virt::PagePerm, task::executor::SwitchContext},
    conf,
    error::{InternalError, Result},
    mm::{
        phys::PhysFrame,
        virt::{VirtAddr, KERN_PAGE_TABLE},
    },
    util::{priority::PriorityQueueWithLock, serial_id::SerialIdGenerator, stack::StackAllocator},
};

use super::{runtime, Task, TaskId, TaskPriority};

type TaskQueue = PriorityQueueWithLock<TaskPriority, TaskId>;

static EXECUTOR_STACK_ALLOCATOR: StackAllocator = StackAllocator::new(
    VirtAddr::new(arch::layout::EXECUTOR_STACK_LIMIT),
    arch::layout::EXECUTOR_STACK_SIZE,
    conf::PAGE_SIZE,
    |addr, size| {
        let mut page_table = KERN_PAGE_TABLE.write();
        for i in (0..size).step_by(conf::PAGE_SIZE) {
            let virt_addr = addr + i;
            let phys_frame = PhysFrame::alloc()?;
            page_table.map(
                virt_addr,
                phys_frame,
                PagePerm::G | PagePerm::R | PagePerm::W,
            )?;
        }
        Ok(())
    },
    |addr, size| {
        let mut page_table = KERN_PAGE_TABLE.write();
        for i in (0..size).step_by(conf::PAGE_SIZE) {
            let virt_addr = addr + i;
            page_table.unmap(virt_addr)?;
        }
        Ok(())
    },
);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ExecutorId(u64);

impl ExecutorId {
    fn new() -> Self {
        static ID_GENERATOR: Mutex<SerialIdGenerator> = Mutex::new(SerialIdGenerator::new());

        Self(ID_GENERATOR.lock().generate())
    }
}

impl Display for ExecutorId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ExecutorPriority(u16);

impl ExecutorPriority {
    pub const fn new(priority: u16) -> Self {
        Self(priority)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutorBehaviorOnNoTask {
    IDLE,
    EXIT,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutorStatus {
    Running,
    Finished,
}

pub struct Executor {
    id: ExecutorId,
    priority: ExecutorPriority,
    beh_on_no_task: ExecutorBehaviorOnNoTask,
    status: ExecutorStatus,
    stack_top: VirtAddr,
    switch_context: SwitchContext,
    task_registry: BTreeMap<TaskId, Task>,
    task_queue: Arc<TaskQueue>,
    task_waker: BTreeMap<TaskId, Waker>,
}

impl Executor {
    pub fn new(
        priority: ExecutorPriority,
        beh_on_no_task: ExecutorBehaviorOnNoTask,
    ) -> Pin<Box<Self>> {
        let entry = VirtAddr::new(arch::task::executor::launch as usize);
        let stack_top = EXECUTOR_STACK_ALLOCATOR.alloc().unwrap();

        let executor = Box::pin(Self {
            id: ExecutorId::new(),
            priority,
            beh_on_no_task,
            status: ExecutorStatus::Running,
            stack_top,
            switch_context: SwitchContext::new_executor(
                entry,
                stack_top - core::mem::size_of::<usize>(),
            ),
            task_registry: BTreeMap::new(),
            task_queue: Arc::new(TaskQueue::new()),
            task_waker: BTreeMap::new(),
        });

        unsafe {
            arch::mm::push_stack(stack_top, &*executor.as_ref() as *const _ as usize);
        }

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

    pub fn switch_context_addr(&self) -> VirtAddr {
        VirtAddr::new(&self.switch_context as *const _ as usize)
    }

    pub fn spawn(&mut self, task: Task) -> Result<&mut Self> {
        let id = task.id;
        self.task_queue.add(id, task.priority);
        self.task_registry
            .try_insert(id, task)
            .map_err(|_| InternalError::DuplicateTaskId)?;
        Ok(self)
    }

    pub fn run(&mut self) {
        let Self {
            task_registry,
            task_queue,
            task_waker,
            ..
        } = self;

        loop {
            while let Some(task_id) = task_queue.pop() {
                let task = match task_registry.get_mut(&task_id) {
                    Some(task) => task,
                    None => continue,
                };

                let waker = task_waker
                    .entry(task_id)
                    .or_insert_with(|| TaskWaker::new(task.id, task.priority, task_queue.clone()));

                let mut context = Context::from_waker(waker);
                match task.poll(&mut context) {
                    Poll::Ready(()) => {
                        task_registry.remove(&task_id);
                        task_waker.remove(&task_id);
                    }
                    Poll::Pending => {}
                }
            }

            if self.beh_on_no_task == ExecutorBehaviorOnNoTask::IDLE {
                arch::wait_for_interrupt();
            } else {
                self.status = ExecutorStatus::Finished;
                break;
            }
        }
    }
}

impl Drop for Executor {
    fn drop(&mut self) {
        EXECUTOR_STACK_ALLOCATOR.dealloc(self.stack_top).unwrap();
    }
}

pub extern "C" fn start(address: usize) -> ! {
    let mut executor = unsafe { Box::from_raw(address as *mut Executor) };
    executor.run();

    runtime::switch_yield();
    unreachable!();
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
    fn new(task_id: TaskId, task_priority: TaskPriority, task_queue: Arc<TaskQueue>) -> Waker {
        Waker::from(Arc::new(Self {
            task_id,
            task_priority,
            task_queue,
        }))
    }

    fn wake_task(&self) {
        self.task_queue.add(self.task_id, self.task_priority);
    }
}
