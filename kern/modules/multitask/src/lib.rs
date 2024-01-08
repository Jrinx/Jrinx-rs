#![no_std]
#![feature(asm_const)]
#![feature(iter_map_windows)]
#![feature(map_try_insert)]
#![feature(offset_of)]
#![feature(slice_group_by)]
#![feature(sync_unsafe_cell)]

mod arch;
pub mod executor;
pub mod inspector;
pub mod runtime;

extern crate alloc;
#[macro_use]
extern crate log;
#[macro_use]
extern crate jrinx_hal;

use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use alloc::boxed::Box;
use executor::Executor;
use jrinx_serial_id_macro::SerialId;
use jrinx_util::fastpq::FastPriority;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, SerialId)]
pub struct TaskId(u64);

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TaskPriority(FastPriority);

impl TaskPriority {
    pub const NUM: usize = FastPriority::NUM;
    pub const MAX: u8 = (Self::NUM - 1) as u8;

    pub const fn new(priority: u8) -> Self {
        Self(FastPriority::new(priority))
    }
}

impl From<TaskPriority> for FastPriority {
    fn from(value: TaskPriority) -> Self {
        value.0
    }
}

impl From<u8> for TaskPriority {
    fn from(value: u8) -> Self {
        Self::new(value)
    }
}

pub struct Task {
    id: TaskId,
    priority: TaskPriority,
    future: Pin<Box<dyn Future<Output = ()> + Send + Sync>>,
}

impl Task {
    pub fn new(
        future: impl Future<Output = ()> + Send + Sync + 'static,
        priority: TaskPriority,
    ) -> Self {
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

pub fn do_spawn(future: impl Future<Output = ()> + Send + Sync + 'static, priority: TaskPriority) {
    Executor::with_current(|ex| {
        ex.spawn(Task::new(future, priority)).unwrap();
    })
    .unwrap();
}

#[macro_export]
macro_rules! spawn {
    ($future: expr) => {
        $crate::do_spawn($future, $crate::TaskPriority::default())
    };
    (pri := $priority:expr => $future: expr) => {
        $crate::do_spawn($future, $priority.into())
    };
}

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

#[macro_export]
macro_rules! yield_now {
    () => {
        $crate::do_yield().await
    };
}
