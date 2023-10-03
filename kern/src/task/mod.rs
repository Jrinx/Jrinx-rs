pub mod executor;

use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use alloc::boxed::Box;

use crate::{cpudata, util::identity::IdGenerater};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TaskId(u64);

impl IdGenerater for TaskId {}

impl TaskId {
    fn new() -> Self {
        Self(Self::generate())
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TaskPriority(u16);

impl TaskPriority {
    pub const fn new(priority: u16) -> Self {
        Self(priority)
    }
}

pub struct Task {
    id: TaskId,
    priority: TaskPriority,
    future: Pin<Box<dyn Future<Output = ()>>>,
}

impl Task {
    pub fn new(future: impl Future<Output = ()> + 'static, priority: TaskPriority) -> Self {
        Self {
            id: TaskId::new(),
            priority,
            future: Box::pin(future),
        }
    }

    pub fn poll(&mut self, cx: &mut Context) -> Poll<()> {
        self.future.as_mut().poll(cx)
    }
}

struct YieldNow {
    done: bool,
}

impl YieldNow {
    fn new() -> Self {
        Self { done: false }
    }
}

impl Future for YieldNow {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.done {
            Poll::Ready(())
        } else {
            self.done = true;
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

pub fn spawn(future: impl Future<Output = ()> + 'static) {
    spawn_with_priority(future, TaskPriority::default());
}

pub fn spawn_with_priority(future: impl Future<Output = ()> + 'static, priority: TaskPriority) {
    cpudata::with_cpu_executor(|executor| {
        executor.spawn(Task::new(future, priority)).unwrap();
    });
}

pub async fn yield_now() {
    YieldNow::new().await;
}
