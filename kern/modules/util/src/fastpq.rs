use alloc::collections::VecDeque;
use spin::Mutex;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct FastPriority(u8);

impl FastPriority {
    pub const NUM: usize = 64;
    pub const MAX: u8 = (Self::NUM - 1) as u8;

    pub const fn new(priority: u8) -> Self {
        debug_assert!(priority <= Self::MAX);

        Self(priority)
    }
}

pub struct FastPriorityQueue<P: Clone + Copy + Into<FastPriority>, I> {
    bits: u64,
    queues: [VecDeque<(P, I)>; 64],
}

impl<P: Clone + Copy + Into<FastPriority>, I> FastPriorityQueue<P, I> {
    const INIT_VAL: VecDeque<(P, I)> = VecDeque::new();

    pub const fn new() -> Self {
        Self {
            bits: 0,
            queues: [Self::INIT_VAL; FastPriority::NUM],
        }
    }

    pub fn enqueue(&mut self, priority: P, item: I) {
        let pri = priority.into().0;
        let bit = 1 << pri;
        self.bits |= bit;
        self.queues[pri as usize].push_back((priority, item));
    }

    pub fn dequeue(&mut self) -> Option<(P, I)> {
        if self.bits == 0 {
            return None;
        }
        let highest_priority = FastPriority::MAX - self.bits.leading_zeros() as u8;
        self.queues[highest_priority as usize]
            .pop_front()
            .map(|result| {
                if self.queues[highest_priority as usize].is_empty() {
                    self.bits &= !(1 << highest_priority);
                }
                result
            })
    }
}

pub struct FastPriorityQueueWithLock<P: Clone + Copy + Into<FastPriority>, I> {
    inner: Mutex<FastPriorityQueue<P, I>>,
}

impl<P: Clone + Copy + Into<FastPriority>, I> FastPriorityQueueWithLock<P, I> {
    pub const fn new() -> Self {
        Self {
            inner: Mutex::new(FastPriorityQueue::new()),
        }
    }

    pub fn enqueue(&self, priority: P, item: I) {
        self.inner.lock().enqueue(priority, item);
    }

    pub fn dequeue(&self) -> Option<(P, I)> {
        self.inner.lock().dequeue()
    }
}
