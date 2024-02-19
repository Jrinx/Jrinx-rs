#![no_std]

extern crate alloc;

use alloc::{boxed::Box, collections::VecDeque};
use jrinx_hal::{hal, Cpu, Hal, Interrupt};
use jrinx_percpu::percpu;
use spin::Mutex;

pub struct IntercpuEvent {
    handler: Box<dyn FnOnce() + Send + 'static>,
}

#[percpu]
static INTERCPU_EVENT_QUEUE: Mutex<IntercpuEventQueue> = Mutex::new(IntercpuEventQueue::new());

pub fn with_current<F, R>(f: F) -> R
where
    F: FnOnce(&mut IntercpuEventQueue) -> R,
{
    hal!()
        .interrupt()
        .with_saved_off(|| f(&mut INTERCPU_EVENT_QUEUE.as_ref().lock()))
}

impl IntercpuEvent {
    pub fn create(cpu_id: usize, handler: impl FnOnce() + Send + 'static) {
        if cpu_id == hal!().cpu().id() {
            handler();
        } else {
            INTERCPU_EVENT_QUEUE.with_spec_ref(cpu_id, |queue| {
                queue.lock().add(Self {
                    handler: Box::new(handler),
                });
            });
            hal!().interrupt().send_ipi(&[cpu_id]);
        }
    }

    pub fn fire(self) {
        (self.handler)();
    }
}

pub struct IntercpuEventQueue {
    queue: VecDeque<IntercpuEvent>,
}

impl IntercpuEventQueue {
    pub const fn new() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }

    fn add(&mut self, event: IntercpuEvent) {
        self.queue.push_back(event);
    }

    pub fn pop(&mut self) -> Option<IntercpuEvent> {
        self.queue.pop_front()
    }
}
