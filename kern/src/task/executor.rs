use core::{
    pin::Pin,
    task::{Context, Poll, Waker},
};

use alloc::{boxed::Box, collections::BTreeMap, sync::Arc, task::Wake};

use crate::{
    arch::{self, mm::virt::PagePerm},
    conf,
    error::{HaltReason, InternalError, Result},
    mm::{
        phys::PhysFrame,
        virt::{VirtAddr, KERN_PAGE_TABLE},
    },
    util::{priority::PriorityQueueWithLock, stack::StackAllocator},
};

use super::{Task, TaskId, TaskPriority};

type TaskQueue = PriorityQueueWithLock<TaskPriority, TaskId>;

static EXECUTOR_STACK_ALLOCATOR: StackAllocator = StackAllocator::new(
    VirtAddr::new(arch::layout::EXECUTOR_STACK_LIMIT),
    arch::layout::EXECUTOR_STACK_SIZE,
    conf::PAGE_SIZE,
);

pub struct Executor {
    stack_top: VirtAddr,
    task_registry: BTreeMap<TaskId, Task>,
    task_queue: Arc<TaskQueue>,
    task_waker: BTreeMap<TaskId, Waker>,
}

impl Executor {
    pub fn new() -> Pin<Box<Self>> {
        let stack_top = Self::setup_vm().unwrap();

        let executor = Self {
            stack_top,
            task_registry: BTreeMap::new(),
            task_queue: Arc::new(TaskQueue::new()),
            task_waker: BTreeMap::new(),
        };

        Box::pin(executor)
    }

    pub fn addr(&self) -> VirtAddr {
        VirtAddr::new(self as *const _ as usize)
    }

    pub fn stack_top(&self) -> VirtAddr {
        self.stack_top
    }

    pub fn spawn(&mut self, task: Task) -> Result<&mut Self> {
        let id = task.id;
        self.task_queue.add(id, task.priority);
        self.task_registry
            .try_insert(id, task)
            .map_err(|_| InternalError::DuplicateTaskId)?;
        Ok(self)
    }

    pub fn run(&mut self) -> ! {
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

            // TODO: Wait for interrupt?
            arch::halt(HaltReason::NormalExit);
        }
    }

    fn setup_vm() -> Result<VirtAddr> {
        let stack_top = EXECUTOR_STACK_ALLOCATOR.alloc();

        let mut page_table = KERN_PAGE_TABLE.write();
        for i in (0..arch::layout::EXECUTOR_STACK_SIZE).step_by(conf::PAGE_SIZE) {
            let virt_addr = stack_top - i - conf::PAGE_SIZE;
            let phys_frame = PhysFrame::alloc()?;
            page_table.map(
                virt_addr,
                phys_frame,
                PagePerm::G | PagePerm::R | PagePerm::W,
            )?;
        }
        Ok(stack_top)
    }
}

impl Drop for Executor {
    fn drop(&mut self) {
        EXECUTOR_STACK_ALLOCATOR.dealloc(self.stack_top).unwrap();
    }
}

pub extern "C" fn start(address: usize) -> ! {
    let executor = unsafe { &mut *(address as *mut Executor) };
    executor.run();
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
