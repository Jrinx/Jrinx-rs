use alloc::collections::{BTreeMap, VecDeque};
use spin::RwLock;

pub trait Priority = Clone + Copy + Ord;

pub struct PriorityQueue<P, I> {
    queue: BTreeMap<P, VecDeque<I>>,
}

impl<P, I> PriorityQueue<P, I>
where
    P: Priority,
{
    pub const fn new() -> Self {
        Self {
            queue: BTreeMap::new(),
        }
    }

    pub fn add(&mut self, item: I, priority: P) {
        self.queue
            .entry(priority)
            .or_insert_with(VecDeque::new)
            .push_back(item);
    }

    pub fn pop(&mut self) -> Option<I> {
        let (priority, mut queue) = self.queue.pop_last()?;
        let item = queue.pop_front()?;
        if !queue.is_empty() {
            self.queue.insert(priority, queue);
        }
        Some(item)
    }
}

pub struct PriorityQueueWithLock<P, I> {
    inner: RwLock<PriorityQueue<P, I>>,
}

impl<P, I> PriorityQueueWithLock<P, I>
where
    P: Priority,
{
    pub const fn new() -> Self {
        Self {
            inner: RwLock::new(PriorityQueue::new()),
        }
    }

    pub fn add(&self, item: I, priority: P) {
        self.inner.write().add(item, priority);
    }

    pub fn pop(&self) -> Option<I> {
        self.inner.write().pop()
    }
}
