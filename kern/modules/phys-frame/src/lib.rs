#![no_std]
#![feature(allocator_api)]

extern crate alloc;

use alloc::{alloc::Global, sync::Arc};
use jrinx_addr::{PhysAddr, VirtAddr};
use jrinx_error::{InternalError, Result};

use core::{
    alloc::{Allocator, Layout},
    ptr::NonNull,
};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysFrame<A: Allocator = Global> {
    addr: PhysAddr,
    alloc: A,
}

#[repr(C, align(4096))]
struct PhysFrameMemory([u8; jrinx_config::PAGE_SIZE]);

const PHYS_FRAME_MEMORY_LAYOUT: Layout = Layout::new::<PhysFrameMemory>();

impl<A: Allocator> Drop for PhysFrame<A> {
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
}

impl<A: Allocator> PhysFrame<A> {
    pub fn alloc_in(alloc: A) -> Result<Arc<Self>> {
        let addr: NonNull<u8> =
            core::hint::black_box(alloc.allocate_zeroed(PHYS_FRAME_MEMORY_LAYOUT))
                .map_err(|_| InternalError::NotEnoughMem)?
                .cast();

        let frame = Self {
            addr: VirtAddr::new(addr.as_ptr() as usize).to_phys(),
            alloc,
        };

        Ok(Arc::new(frame))
    }

    pub fn addr(&self) -> PhysAddr {
        self.addr
    }
}
