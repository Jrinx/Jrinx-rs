use core::{cmp::Reverse, fmt::Debug, time::Duration};

use alloc::{
    boxed::Box,
    collections::{BTreeMap, BinaryHeap},
    sync::Arc,
};
use spin::Mutex;

use crate::{
    arch, cpudata,
    error::{InternalError, Result},
    util::serial_id::SerialIdGenerator,
};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
struct TimedEventId(u64);

impl TimedEventId {
    fn new() -> Self {
        static ID_GENERATOR: Mutex<SerialIdGenerator> = Mutex::new(SerialIdGenerator::new());

        Self(ID_GENERATOR.lock().generate())
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum TimedEventStatus {
    Pending,
    Timeout,
    Cancelled,
}

pub struct TimedEvent {
    id: TimedEventId,
    cpu_id: usize,
    time: Duration,
    status: TimedEventStatus,
    timeout_handler: Option<Box<dyn FnOnce()>>,
    cancel_handler: Option<Box<dyn FnOnce()>>,
}

impl TimedEvent {
    pub fn new(
        time: Duration,
        timeout_handler: impl FnOnce() + 'static,
        cancel_handler: impl FnOnce() + 'static,
    ) -> TimedEventTracker {
        let tracker = TimedEventTracker(Arc::new(Mutex::new(Self {
            id: TimedEventId::new(),
            cpu_id: arch::cpu::id(),
            time,
            status: TimedEventStatus::Pending,
            timeout_handler: Some(Box::new(timeout_handler)),
            cancel_handler: Some(Box::new(cancel_handler)),
        })));
        cpudata::with_cpu_timed_event_queue(|queue| queue.add(tracker.clone())).unwrap();
        tracker
    }

    fn invoke(&mut self, target_status: TimedEventStatus) -> Result<()> {
        if target_status == TimedEventStatus::Pending || self.status != TimedEventStatus::Pending {
            return Err(InternalError::InvalidTimedEventStatus);
        }
        self.status = target_status;
        match target_status {
            TimedEventStatus::Timeout => {
                self.timeout_handler
                    .take()
                    .ok_or(InternalError::InvalidTimedEventStatus)?()
            }
            TimedEventStatus::Cancelled => {
                self.cancel_handler
                    .take()
                    .ok_or(InternalError::InvalidTimedEventStatus)?()
            }
            _ => unreachable!(),
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct TimedEventTracker(Arc<Mutex<TimedEvent>>);

impl TimedEventTracker {
    pub fn timeout(&self) -> Result<()> {
        cpudata::with_timed_event_queue(self.cpu_id(), |queue| queue.remove(self.clone()))??;
        self.0.lock().invoke(TimedEventStatus::Timeout)
    }

    pub fn cancel(&self) -> Result<()> {
        cpudata::with_timed_event_queue(self.cpu_id(), |queue| queue.remove(self.clone()))??;
        self.0.lock().invoke(TimedEventStatus::Cancelled)
    }

    fn id(&self) -> TimedEventId {
        self.0.lock().id
    }

    fn cpu_id(&self) -> usize {
        self.0.lock().cpu_id
    }

    fn time(&self) -> Duration {
        self.0.lock().time
    }
}

pub struct TimedEventQueue {
    registry: BTreeMap<TimedEventId, TimedEventTracker>,
    queue: BinaryHeap<Reverse<(Duration, TimedEventId)>>,
}

impl TimedEventQueue {
    pub fn new() -> Self {
        Self {
            registry: BTreeMap::new(),
            queue: BinaryHeap::new(),
        }
    }

    pub fn peek_outdated(&self) -> Option<TimedEventTracker> {
        self.peek()
            .filter(|tracker| tracker.time() <= arch::cpu::time())
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
            arch::cpu::set_timer(*time);
        } else {
            arch::cpu::set_timer(Duration::MAX);
        }
    }
}
