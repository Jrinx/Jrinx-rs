pub mod boot;
use core::fmt::Display;

use alloc::string::String;
use bitflags::bitflags;
use jrinx_addr::PhysAddr;

use crate::{common, CloneKernel, GenericPagePerm, GenericPageTableEntry};

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct PagePerm: usize {
        const __V = 1 << 0;
        const __R = 1 << 1;
        const __W = 1 << 2;
        const __X = 1 << 3;
        const __U = 1 << 4;
        const __G = 1 << 5;
        const __A = 1 << 6;
        const __D = 1 << 7;
    }
}

impl GenericPagePerm for PagePerm {
    const V: Self = Self::__V;
    const R: Self = Self::__R;
    const W: Self = Self::__W;
    const X: Self = Self::__X;
    const U: Self = Self::__U;
    const G: Self = Self::__G;
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
    pub(in crate::arch) fn new(phys_addr: PhysAddr, perm: PagePerm) -> Self {
        let mut pte = Self { bits: 0 };
        pte.set(phys_addr, perm);
        pte
    }

    pub(in crate::arch) unsafe fn from_raw(bits: usize) -> Self {
        Self { bits }
    }

    pub fn is_valid(&self) -> bool {
        self.bits & PagePerm::V.bits() != 0
    }
}

impl GenericPageTableEntry<PagePerm> for PageTableEntry {
    fn set(&mut self, phys_addr: PhysAddr, perm: PagePerm) {
        let mut perm = perm;
        if perm.contains(PagePerm::R) {
            perm.insert(PagePerm::__A); // access-bit
        }
        if perm.contains(PagePerm::W) {
            perm.insert(PagePerm::__D); // dirty-bit
        }
        self.bits = (phys_addr.align_page_down().as_usize() >> 2) | perm.bits();
    }

    fn clr(&mut self) {
        self.bits = 0;
    }
}

impl From<PageTableEntry> for (PhysAddr, PagePerm) {
    fn from(value: PageTableEntry) -> Self {
        let phys_addr = PhysAddr::new((value.bits << 2) & !(jrinx_config::PAGE_SIZE - 1));
        let perm = PagePerm::from_bits_truncate(value.bits);
        (phys_addr, perm)
    }
}

impl From<PageTableEntry> for usize {
    fn from(value: PageTableEntry) -> Self {
        value.bits
    }
}

impl CloneKernel for common::PageTable {
    fn clone_kernel(dst: &mut [usize]) {
        boot::BootPageTable::clone_into(dst)
    }
}
