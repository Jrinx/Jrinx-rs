use core::fmt::Display;

use alloc::string::String;
use bitflags::bitflags;
use jrinx_addr::{PhysAddr, VirtAddr};

use crate::mm::virt::PageTable;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct PagePerm: usize {
        const V = 1 << 0;
        const R = 1 << 1;
        const W = 1 << 2;
        const X = 1 << 3;
        const U = 1 << 4;
        const G = 1 << 5;
        const A = 1 << 6;
        const D = 1 << 7;
    }
}

impl Display for PagePerm {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut s = String::from("VRWXUG");
        for i in 0..6 {
            if self.bits() & (1 << i) == 0 {
                s.replace_range(i..=i, "-");
            }
        }
        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct PageTableEntry {
    bits: usize,
}

impl PageTableEntry {
    pub fn set(&mut self, phys_addr: PhysAddr, perm: PagePerm) {
        let mut perm = perm;
        if perm.contains(PagePerm::R) {
            perm.insert(PagePerm::A); // access-bit
        }
        if perm.contains(PagePerm::W) {
            perm.insert(PagePerm::D); // dirty-bit
        }
        self.bits = (phys_addr.align_page_down().as_usize() >> 2) | perm.bits();
    }

    pub fn clr(&mut self) {
        self.bits = 0;
    }

    pub fn as_tuple(&self) -> (PhysAddr, PagePerm) {
        let phys_addr = PhysAddr::new((self.bits << 2) & !(jrinx_config::PAGE_SIZE - 1));
        let perm = PagePerm::from_bits_truncate(self.bits);
        (phys_addr, perm)
    }

    pub fn is_valid(&self) -> bool {
        self.bits & PagePerm::V.bits() != 0
    }
}

pub fn enable_pt_mapping(page_table: &PageTable) {
    let pt_ppn = page_table.addr().as_usize() / jrinx_config::PAGE_SIZE;
    unsafe {
        #[cfg(target_arch = "riscv64")]
        riscv::register::satp::set(riscv::register::satp::Mode::Sv39, 0, pt_ppn);

        #[cfg(target_arch = "riscv32")]
        riscv::register::satp::set(riscv::register::satp::Mode::Sv32, 0, pt_ppn);

        riscv::asm::sfence_vma_all();
    }
}

pub fn sync(addr: VirtAddr) {
    unsafe {
        core::arch::asm!("sfence.vma {}, x0", in(reg) addr.as_usize());
    }
}
