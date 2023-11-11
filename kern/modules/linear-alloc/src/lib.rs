#![no_std]

extern crate alloc;

use core::sync::atomic::{AtomicUsize, Ordering};

use alloc::collections::{BTreeSet, VecDeque};
use jrinx_addr::VirtAddr;
use jrinx_error::{InternalError, Result};
use spin::Mutex;

type MapperOrUnmapperFn = fn(VirtAddr, usize) -> Result<()>;

pub struct LinearAllocator {
    region: (VirtAddr, usize),
    ele_size: usize,
    grd_size: usize,
    next_va: AtomicUsize,
    allocated_va: Mutex<BTreeSet<VirtAddr>>,
    recycled_va: Mutex<VecDeque<VirtAddr>>,
    map_fn: MapperOrUnmapperFn,
    unmap_fn: MapperOrUnmapperFn,
}

impl LinearAllocator {
    pub const fn new(
        region: (VirtAddr, usize),
        ele_size: usize,
        grd_size: usize,
        map_fn: MapperOrUnmapperFn,
        unmap_fn: MapperOrUnmapperFn,
    ) -> Self {
        Self {
            region,
            ele_size,
            grd_size,
            next_va: AtomicUsize::new(region.0.as_usize()),
            allocated_va: Mutex::new(BTreeSet::new()),
            recycled_va: Mutex::new(VecDeque::new()),
            map_fn,
            unmap_fn,
        }
    }

    pub fn allocate(&self) -> Result<VirtAddr> {
        let va = match self.recycled_va.lock().pop_front() {
            Some(va) => va,
            None => match self.next_va.load(Ordering::SeqCst) {
                va if va + self.ele_size > (self.region.0 + self.region.1).as_usize() => {
                    Err(InternalError::NotEnoughMem)
                }
                _ => Ok(self
                    .next_va
                    .fetch_add(self.ele_size + self.grd_size, Ordering::SeqCst)),
            }
            .map(VirtAddr::new)?,
        };
        self.allocated_va.lock().insert(va);

        (self.map_fn)(va, self.ele_size)?;

        Ok(va)
    }

    pub fn deallocate(&self, va: VirtAddr) -> Result<()> {
        let va = if !self.allocated_va.lock().remove(&va) {
            Err(InternalError::InvalidVirtAddr)
        } else {
            self.recycled_va.lock().push_back(va);
            Ok(va)
        }?;

        (self.unmap_fn)(va, self.ele_size)
    }
}
