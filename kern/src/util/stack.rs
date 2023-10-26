use core::sync::atomic::{AtomicUsize, Ordering};

use alloc::collections::{BTreeSet, VecDeque};
use jrinx_error::{InternalError, Result};
use spin::Mutex;

use crate::mm::virt::VirtAddr;

pub struct StackAllocator {
    stack_size: usize,
    guard_size: usize,
    next_top: AtomicUsize,
    allocated_tops: Mutex<BTreeSet<VirtAddr>>,
    recycled_tops: Mutex<VecDeque<VirtAddr>>,
    vm_map: fn(VirtAddr, usize) -> Result<()>,
    vm_unmap: fn(VirtAddr, usize) -> Result<()>,
}

impl StackAllocator {
    pub const fn new(
        limit: VirtAddr,
        stack_size: usize,
        guard_size: usize,
        map: fn(VirtAddr, usize) -> Result<()>,
        unmap: fn(VirtAddr, usize) -> Result<()>,
    ) -> Self {
        Self {
            stack_size,
            guard_size,
            next_top: AtomicUsize::new(limit.as_usize()),
            allocated_tops: Mutex::new(BTreeSet::new()),
            recycled_tops: Mutex::new(VecDeque::new()),
            vm_map: map,
            vm_unmap: unmap,
        }
    }

    pub fn alloc(&self) -> Result<VirtAddr> {
        let stack_top = if let Some(top) = self.recycled_tops.lock().pop_front() {
            top
        } else {
            let stack_top = VirtAddr::new(
                self.next_top
                    .fetch_sub(self.stack_size + self.guard_size, Ordering::SeqCst),
            );
            self.allocated_tops.lock().insert(stack_top);
            stack_top
        };

        (self.vm_map)(stack_top - self.stack_size, self.stack_size)?;

        Ok(stack_top)
    }

    pub fn dealloc(&self, top: VirtAddr) -> Result<()> {
        let stack_top = if !self.allocated_tops.lock().remove(&top) {
            Err(InternalError::InvalidVirtAddr)
        } else {
            self.recycled_tops.lock().push_back(top);
            Ok(top)
        }?;

        (self.vm_unmap)(stack_top - self.stack_size, self.stack_size)?;

        Ok(())
    }
}
