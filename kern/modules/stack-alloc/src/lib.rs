#![no_std]
#![feature(trait_alias)]

use alloc::{
    boxed::Box,
    collections::{BTreeMap, VecDeque},
};
use core::sync::atomic::AtomicUsize;
use spin::Mutex;

use jrinx_addr::VirtAddr;
use jrinx_config::PAGE_SIZE;
use jrinx_error::{InternalError, Result};

extern crate alloc;

pub trait MapperOrUnmapperFn = Fn(VirtAddr) -> Result<()> + Send + Sync;

pub struct StackAllocator {
    region: (VirtAddr, usize),
    guard_size: usize,
    next: AtomicUsize,
    allocated: Mutex<BTreeMap<VirtAddr, usize>>,
    cached: Mutex<BTreeMap<usize, VecDeque<VirtAddr>>>,
    map: Box<dyn MapperOrUnmapperFn>,
    unmap: Box<dyn MapperOrUnmapperFn>,
}

impl StackAllocator {
    pub fn new(
        region: (VirtAddr, usize),
        guard_size: usize,
        map: impl MapperOrUnmapperFn + 'static,
        unmap: impl MapperOrUnmapperFn + 'static,
    ) -> Self {
        let guard_size = guard_size.next_multiple_of(PAGE_SIZE);
        Self {
            region,
            guard_size,
            next: AtomicUsize::new(region.0.as_usize() - guard_size),
            allocated: Mutex::new(BTreeMap::new()),
            cached: Mutex::new(BTreeMap::new()),
            map: Box::new(map),
            unmap: Box::new(unmap),
        }
    }

    pub fn allocate(&self, size: usize) -> Result<VirtAddr> {
        let size = size.next_multiple_of(PAGE_SIZE);

        let va = match self.cached.lock().entry(size).or_default().pop_front() {
            Some(cached) => cached,
            None => match self.next.load(core::sync::atomic::Ordering::Acquire) {
                va if va + size > (self.region.0 + self.region.1).as_usize() => {
                    Err(InternalError::NotEnoughMem)
                }
                _ => Ok(self.next.fetch_add(
                    size + self.guard_size,
                    core::sync::atomic::Ordering::Release,
                )),
            }
            .map(VirtAddr::new)?,
        };
        let stack_top = va + size + self.guard_size;
        self.allocated.lock().insert(stack_top, size);

        for i in (0..size).step_by(PAGE_SIZE) {
            (self.map)(va + self.guard_size + i)?;
        }

        Ok(stack_top)
    }

    pub fn deallocate(&self, stack_top: VirtAddr) -> Result<()> {
        let size = self
            .allocated
            .lock()
            .remove(&stack_top)
            .ok_or(InternalError::InvalidVirtAddr)?;

        let va = stack_top - size - self.guard_size;

        for i in (0..size).step_by(PAGE_SIZE) {
            (self.unmap)(va + self.guard_size + i)?;
        }

        self.cached.lock().entry(size).or_default().push_front(va);
        Ok(())
    }
}
