#![no_std]
#![feature(const_binary_heap_constructor)]

extern crate alloc;

use core::{cmp::Reverse, fmt::Debug, time::Duration};

use alloc::{
    boxed::Box,
    collections::{BTreeMap, BinaryHeap},
    sync::Arc,
};
use jrinx_error::{InternalError, Result};
use jrinx_hal::{hal, Cpu, Hal, Interrupt};
use jrinx_percpu::percpu;
use jrinx_serial_id_macro::SerialId;
use spin::Mutex;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, SerialId)]
struct TimedEventId(u64);

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum TimedEventStatus {
    Pending,
    Timeout,
    Cancelled,
}

pub struct TimedEventHandler {
    timeout: Box<dyn FnOnce() + Send + 'static>,
    cancel: Box<dyn FnOnce() + Send + 'static>,
}

impl TimedEventHandler {
    pub fn new(
        timeout: impl FnOnce() + Send + 'static,
        cancel: impl FnOnce() + Send + 'static,
    ) -> Self {
        Self {
            timeout: Box::new(timeout),
            cancel: Box::new(cancel),
        }
    }
}

pub struct TimedEvent {
    id: TimedEventId,
    cpu_id: usize,
    time: Duration,
    inner: Mutex<TimedEventInner>,
}

struct TimedEventInner {
    status: TimedEventStatus,
    handler: Option<TimedEventHandler>,
}

#[percpu]
static TIMED_EVENT_QUEUE: Mutex<TimedEventQueue> = Mutex::new(TimedEventQueue::new());

pub fn with_current<F, R>(f: F) -> R
where
    F: FnOnce(&mut TimedEventQueue) -> R,
{
    hal!()
        .interrupt()
        .with_saved_off(|| f(&mut TIMED_EVENT_QUEUE.as_ref().lock()))
}

impl TimedEvent {
    pub fn create(time: Duration, handler: TimedEventHandler) -> TimedEventTracker {
        let tracker = TimedEventTracker(Arc::new(Self {
            id: TimedEventId::new(),
            cpu_id: hal!().cpu().id(),
            time,
            inner: Mutex::new(TimedEventInner {
                status: TimedEventStatus::Pending,
                handler: Some(handler),
            }),
        }));
        TIMED_EVENT_QUEUE.as_ref().lock().add(tracker.clone());
        tracker
    }

    fn invoke(&self, target: TimedEventStatus) -> Result<()> {
        let func = {
            let mut inner = self.inner.lock();
            let handler = inner
                .handler
                .take()
                .ok_or(InternalError::InvalidTimedEventStatus)?;
            let func = match target {
                TimedEventStatus::Timeout => handler.timeout,
                TimedEventStatus::Cancelled => handler.cancel,
                _ => panic!("Invalid timed event status"),
            };
            inner.status = target;
            func
        };
        func();
        Ok(())
    }
}

#[derive(Clone)]
pub struct TimedEventTracker(Arc<TimedEvent>);

impl TimedEventTracker {
    pub fn timeout(&self) -> Result<()> {
        TIMED_EVENT_QUEUE
            .with_spec_ref(self.cpu_id(), |queue| queue.lock().remove(self.clone()))?;
        self.0.invoke(TimedEventStatus::Timeout)
    }

    pub fn cancel(&self) -> Result<()> {
        TIMED_EVENT_QUEUE
            .with_spec_ref(self.cpu_id(), |queue| queue.lock().remove(self.clone()))?;
        self.0.invoke(TimedEventStatus::Cancelled)
    }

    pub fn retired(&self) -> bool {
        self.0.inner.lock().status != TimedEventStatus::Pending
    }

    fn id(&self) -> TimedEventId {
        self.0.id
    }

    fn cpu_id(&self) -> usize {
        self.0.cpu_id
    }

    fn time(&self) -> Duration {
        self.0.time
    }
}

pub struct TimedEventQueue {
    registry: BTreeMap<TimedEventId, TimedEventTracker>,
    queue: BinaryHeap<Reverse<(Duration, TimedEventId)>>,
}

impl TimedEventQueue {
    pub const fn new() -> Self {
        Self {
            registry: BTreeMap::new(),
            queue: BinaryHeap::new(),
        }
    }

    pub fn peek_outdated(&self) -> Option<TimedEventTracker> {
        self.peek()
            .filter(|tracker| tracker.time() <= hal!().cpu().get_time())
    }

    fn add(&mut self, tracker: TimedEventTracker) {
        let id = tracker.id();
        let time = tracker.time();
        self.registry.insert(id, tracker);
        self.queue.push(Reverse((time, id)));
        self.update_timer();
    }

    fn peek(&self) -> Option<TimedEventTracker> {
        self.queue
            .peek()
            .map(|Reverse((_, id))| self.registry.get(id).unwrap().clone())
    }

    fn remove(&mut self, tracker: TimedEventTracker) -> Result<()> {
        let id = tracker.id();
        self.registry
            .remove(&id)
            .ok_or(InternalError::InvalidTimedEventStatus)?;
        self.queue.retain(|&Reverse((_, that_id))| that_id != id);
        self.update_timer();
        Ok(())
    }

    fn update_timer(&self) {
        if let Some(Reverse((time, _))) = self.queue.peek() {
            hal!().cpu().set_timer(*time);
        } else {
            hal!().cpu().set_timer(Duration::MAX);
        }
    }
}
