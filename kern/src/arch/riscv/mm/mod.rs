use alloc::{vec, vec::Vec};

use crate::{
    conf,
    mm::{phys::PhysAddr, virt::VirtAddr},
};

pub mod virt;

pub fn virt_to_phys(addr: VirtAddr) -> PhysAddr {
    PhysAddr::new(addr.as_usize())
}

pub fn phys_to_virt(addr: PhysAddr) -> VirtAddr {
    VirtAddr::new(addr.as_usize())
}

pub unsafe fn push_stack<T>(stack_top: VirtAddr, val: T) -> VirtAddr {
    let stack_ptr = stack_top.as_usize() as *mut T;
    VirtAddr::new({
        let ptr = stack_ptr.sub(1);
        ptr.write(val);
        ptr as usize
    })
}

pub fn get_protected_mem_regions() -> Vec<(VirtAddr, usize)> {
    const BASE_ADDR: usize = 0x80000000;
    vec![(VirtAddr::new(BASE_ADDR), conf::layout::_end() - BASE_ADDR)]
}

pub fn ibar() {
    unsafe {
        core::arch::asm!("fence.i");
    }
}
