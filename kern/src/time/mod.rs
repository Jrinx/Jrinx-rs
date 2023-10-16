use core::{cmp::Reverse, time::Duration};

use alloc::{
    boxed::Box,
    collections::{BTreeMap, BinaryHeap},
    sync::Arc,
};
use spin::Mutex;

use crate::{
    arch,
    error::{InternalError, Result},
    util::serial_id::SerialIdGenerator,
};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub struct TimedEventId(u64);

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
    ) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self {
            time,
            id: TimedEventId::new(),
            status: TimedEventStatus::Pending,
            timeout_handler: Some(Box::new(timeout_handler)),
            cancel_handler: Some(Box::new(cancel_handler)),
        }))
    }

    pub fn id(&self) -> TimedEventId {
        self.id
    }

    pub fn timeout(&mut self) -> Result<()> {
        if self.status != TimedEventStatus::Pending {
            return Err(InternalError::InvalidTimedEventStatus);
        } else {
            self.status = TimedEventStatus::Timeout;
        }
        let handler = self
            .timeout_handler
            .take()
            .ok_or(InternalError::InvalidTimedEventStatus)?;
        handler();
        Ok(())
    }

    pub fn cancel(&mut self) -> Result<()> {
        if self.status != TimedEventStatus::Pending {
            return Err(InternalError::InvalidTimedEventStatus);
        } else {
            self.status = TimedEventStatus::Cancelled;
        }
        let handler = self
            .cancel_handler
            .take()
            .ok_or(InternalError::InvalidTimedEventStatus)?;
        handler();
        Ok(())
    }
}

pub struct TimedEventQueue {
    registry: BTreeMap<TimedEventId, Arc<Mutex<TimedEvent>>>,
    queue: BinaryHeap<Reverse<(Duration, TimedEventId)>>,
}

impl TimedEventQueue {
    pub fn new() -> Self {
        Self {
            registry: BTreeMap::new(),
            queue: BinaryHeap::new(),
        }
    }

    pub fn add(&mut self, timed_event: Arc<Mutex<TimedEvent>>) {
        let (id, time) = {
            let timed_event = timed_event.lock();
            (timed_event.id, timed_event.time)
        };
        self.registry.insert(id, timed_event);
        self.queue.push(Reverse((time, id)));
        self.update_timer();
    }

    pub fn peek(&self) -> Option<TimedEventId> {
        self.queue.peek().map(|Reverse((_, id))| *id)
    }

    pub fn timeout(&mut self, timed_event_id: TimedEventId) -> Result<()> {
        let Some(timed_event) = self.registry.remove(&timed_event_id) else {
            return Err(InternalError::InvalidTimedEventId);
        };
        self.queue.retain(|Reverse((_, id))| *id != timed_event_id);
        self.update_timer();
        timed_event.lock().timeout()?;
        Ok(())
    }

    pub fn cancel(&mut self, timed_event_id: TimedEventId) -> Result<()> {
        let Some(timed_event) = self.registry.remove(&timed_event_id) else {
            return Err(InternalError::InvalidTimedEventId);
        };
        self.queue.retain(|Reverse((_, id))| *id != timed_event_id);
        self.update_timer();
        timed_event.lock().cancel()?;
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
