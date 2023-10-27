pub mod executor;
pub mod runtime;

use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use alloc::boxed::Box;
use jrinx_serial_id::SerialIdGenerator;
use jrinx_serial_id_macro::SerialId;

use crate::cpudata::CpuDataVisitor;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, SerialId)]
pub struct TaskId(u64);

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TaskPriority(u16);

impl TaskPriority {
    pub const fn new(priority: u16) -> Self {
        Self(priority)
    }
}

impl From<u16> for TaskPriority {
    fn from(value: u16) -> Self {
        Self::new(value)
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
            id: TaskId::generate(),
            priority,
            future: Box::pin(future),
        }
    }

    pub fn poll(&mut self, cx: &mut Context) -> Poll<()> {
        self.future.as_mut().poll(cx)
    }
}

pub fn do_spawn(future: impl Future<Output = ()> + 'static, priority: TaskPriority) {
    CpuDataVisitor::new()
        .executor(|executor| {
            executor.spawn(Task::new(future, priority)).unwrap();
        })
        .unwrap();
}

macro_rules! spawn {
    ($future: expr) => {
        $crate::task::do_spawn($future, $crate::task::TaskPriority::default())
    };
    (pri := $priority:expr => $future: expr) => {
        $crate::task::do_spawn($future, $priority.into())
    };
}
pub(crate) use spawn;

pub async fn do_yield() {
    struct YieldNow {
        done: bool,
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

    YieldNow { done: false }.await;
}

macro_rules! yield_now {
    () => {
        $crate::task::do_yield().await
    };
}
pub(crate) use yield_now;
