use alloc::{vec, vec::Vec};
use jrinx_addr::{PhysAddr, VirtAddr};

pub mod virt;

pub fn virt_to_phys(addr: VirtAddr) -> PhysAddr {
    PhysAddr::new(addr.as_usize())
}

pub fn phys_to_virt(addr: PhysAddr) -> VirtAddr {
    VirtAddr::new(addr.as_usize())
}

pub fn get_protected_mem_regions() -> Vec<(VirtAddr, usize)> {
    const BASE_ADDR: usize = 0x80000000;
    vec![(VirtAddr::new(BASE_ADDR), jrinx_layout::_end() - BASE_ADDR)]
}

pub fn ibar() {
    unsafe {
        core::arch::asm!("fence.i");
    }
}
