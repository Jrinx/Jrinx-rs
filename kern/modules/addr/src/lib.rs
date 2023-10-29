#![no_std]

extern crate alloc;

use jrinx_addr_macro::Address;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Address)]
pub struct PhysAddr(usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Address)]
pub struct VirtAddr(usize);

impl VirtAddr {
    #[cfg(feature = "pt_level_2")]
    pub fn indexes(self) -> [usize; 2] {
        [(self.0 >> 22) & 0x3ff, (self.0 >> 12) & 0x3ff]
    }

    #[cfg(feature = "pt_level_3")]
    pub fn indexes(self) -> [usize; 3] {
        [
            (self.0 >> 30) & 0x1ff,
            (self.0 >> 21) & 0x1ff,
            (self.0 >> 12) & 0x1ff,
        ]
    }

    pub fn as_array_base<T>(self) -> &'static mut [T] {
        let addr = self.align_page_down().as_usize();
        unsafe {
            alloc::slice::from_raw_parts_mut(
                addr as *mut T,
                jrinx_config::PAGE_SIZE / core::mem::size_of::<T>(),
            )
        }
    }
}
