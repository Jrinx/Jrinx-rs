use core::sync::atomic::{AtomicUsize, Ordering};

use alloc::collections::{BTreeSet, VecDeque};
use spin::Mutex;

use crate::{
    conf,
    error::{InternalError, Result},
    mm::virt::VirtAddr,
};

pub struct StackAllocator {
    stack_size: usize,
    next_top: AtomicUsize,
    allocated_tops: Mutex<BTreeSet<VirtAddr>>,
    recycled_tops: Mutex<VecDeque<VirtAddr>>,
}

impl StackAllocator {
    pub const fn new(limit: VirtAddr, stack_size: usize) -> Self {
        Self {
            stack_size,
            next_top: AtomicUsize::new(limit.as_usize()),
            allocated_tops: Mutex::new(BTreeSet::new()),
            recycled_tops: Mutex::new(VecDeque::new()),
        }
    }

    pub fn alloc(&self) -> VirtAddr {
        if let Some(top) = self.recycled_tops.lock().pop_front() {
            top
        } else {
            let stack_top = VirtAddr::new(
                self.next_top
                    .fetch_sub(self.stack_size + conf::PAGE_SIZE, Ordering::SeqCst),
            );
            self.allocated_tops.lock().insert(stack_top);
            stack_top
        }
    }

    pub fn dealloc(&self, top: VirtAddr) -> Result<()> {
        if !self.allocated_tops.lock().remove(&top) {
            Err(InternalError::InvalidVirtAddr)
        } else {
            self.recycled_tops.lock().push_back(top);
            Ok(())
        }
    }
}
