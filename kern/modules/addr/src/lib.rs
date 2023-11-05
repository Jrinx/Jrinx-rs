#![no_std]

extern crate alloc;

use core::slice::from_raw_parts_mut;

use jrinx_addr_macro::Address;
use jrinx_config::{PAGE_SIZE, REMAP_MEM_OFFSET};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Address)]
pub struct PhysAddr(usize);

impl PhysAddr {
    pub const fn to_virt(self) -> VirtAddr {
        VirtAddr(self.0 + REMAP_MEM_OFFSET)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Address)]
pub struct VirtAddr(usize);

impl VirtAddr {
    #[cfg(feature = "pt_level_2")]
    pub fn indexes(self) -> [usize; 2] {
        [self.0 >> 22 & 0x3ff, self.0 >> 12 & 0x3ff]
    }

    #[cfg(feature = "pt_level_3")]
    pub fn indexes(self) -> [usize; 3] {
        [
            self.0 >> 30 & 0x1ff,
            self.0 >> 21 & 0x1ff,
            self.0 >> 12 & 0x1ff,
        ]
    }

    pub fn as_array_base<T>(self) -> &'static mut [T] {
        unsafe {
            from_raw_parts_mut(
                self.align_page_down().as_usize() as *mut T,
                PAGE_SIZE / core::mem::size_of::<T>(),
            )
        }
    }

    pub const fn to_phys(self) -> PhysAddr {
        PhysAddr(self.0 - REMAP_MEM_OFFSET)
    }
}
