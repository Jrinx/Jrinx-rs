use crate::mm::{phys::PhysAddr, virt::VirtAddr};

pub mod virt;

pub fn virt_to_phys(addr: VirtAddr) -> PhysAddr {
    PhysAddr::new(addr.as_usize())
}

pub fn phys_to_virt(addr: PhysAddr) -> VirtAddr {
    VirtAddr::new(addr.as_usize())
}
