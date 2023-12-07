#![no_std]
#![feature(allocator_api)]
#![feature(trait_alias)]

extern crate alloc;

use alloc::{alloc::Global, sync::Arc};
use jrinx_addr::{PhysAddr, VirtAddr};
use jrinx_error::{InternalError, Result};

use core::{
    alloc::{Allocator, Layout},
    fmt::Debug,
    ptr::NonNull,
};

pub trait PhysFrameAllocator = Allocator + Send + Sync + 'static;

pub struct PhysFrame {
    addr: PhysAddr,
    alloc: Arc<dyn PhysFrameAllocator>,
}

#[repr(C, align(4096))]
struct PhysFrameMemory([u8; jrinx_config::PAGE_SIZE]);

const PHYS_FRAME_MEMORY_LAYOUT: Layout = Layout::new::<PhysFrameMemory>();

impl Debug for PhysFrame {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("PhysFrame")
            .field("addr", &self.addr)
            .finish()
    }
}

impl PartialEq for PhysFrame {
    fn eq(&self, other: &Self) -> bool {
        self.addr == other.addr
    }
}

impl Eq for PhysFrame {}

impl PartialOrd for PhysFrame {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PhysFrame {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.addr.cmp(&other.addr)
    }
}

impl Drop for PhysFrame {
    fn drop(&mut self) {
        unsafe {
            self.alloc.deallocate(
                NonNull::new(self.addr().to_virt().as_usize() as *mut u8).unwrap(),
                PHYS_FRAME_MEMORY_LAYOUT,
            );
        }
    }
}

impl PhysFrame {
    pub fn alloc() -> Result<Arc<Self>> {
        Self::alloc_in(Global)
    }

    pub fn alloc_in(alloc: impl PhysFrameAllocator) -> Result<Arc<Self>> {
        let addr: NonNull<u8> =
            core::hint::black_box(alloc.allocate_zeroed(PHYS_FRAME_MEMORY_LAYOUT))
                .map_err(|_| InternalError::NotEnoughMem)?
                .cast();

        let frame = Self {
            addr: VirtAddr::new(addr.as_ptr() as usize).to_phys(),
            alloc: Arc::new(alloc),
        };

        Ok(Arc::new(frame))
    }

    pub fn addr(&self) -> PhysAddr {
        self.addr
    }
}
