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
pub struct PhysFrame {
    addr: PhysAddr,
}

#[repr(C, align(4096))]
struct PhysFrameMemory([u8; jrinx_config::PAGE_SIZE]);

const PHYS_FRAME_MEMORY_LAYOUT: Layout = Layout::new::<PhysFrameMemory>();

impl Drop for PhysFrame {
    fn drop(&mut self) {
        unsafe {
            Global.deallocate(
                NonNull::new(self.addr().to_virt().as_usize() as *mut u8).unwrap(),
                PHYS_FRAME_MEMORY_LAYOUT,
            );
        }
    }
}

impl PhysFrame {
    pub fn alloc() -> Result<Arc<Self>> {
        let addr: NonNull<u8> =
            core::hint::black_box(Global.allocate_zeroed(PHYS_FRAME_MEMORY_LAYOUT))
                .map_err(|_| InternalError::NotEnoughMem)?
                .cast();
        Ok(Arc::new(Self {
            addr: VirtAddr::new(addr.as_ptr() as usize).to_phys(),
        }))
    }

    pub fn addr(&self) -> PhysAddr {
        self.addr
    }
}
