use core::{
    fmt::Display,
    ops::{Add, Sub},
};

use alloc::{collections::BTreeMap, sync::Arc};
use jrinx_error::{InternalError, Result};
use lazy_static::lazy_static;
use spin::RwLock;

use crate::{
    arch::{
        self,
        mm::virt::{PagePerm, PageTableEntry},
    },
    mm::phys,
};

use super::phys::{PhysAddr, PhysFrame};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct VirtAddr(usize);

impl Add<usize> for VirtAddr {
    type Output = Self;

    fn add(self, rhs: usize) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl Sub<usize> for VirtAddr {
    type Output = Self;

    fn sub(self, rhs: usize) -> Self::Output {
        Self(self.0 - rhs)
    }
}

impl Sub<VirtAddr> for VirtAddr {
    type Output = usize;

    fn sub(self, rhs: VirtAddr) -> Self::Output {
        self.0 - rhs.0
    }
}

impl Display for VirtAddr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "0x{:x}", self.0)
    }
}

impl VirtAddr {
    pub const fn new(addr: usize) -> Self {
        Self(addr)
    }

    pub const fn as_usize(self) -> usize {
        self.0
    }

    pub fn align_page_down(self) -> Self {
        Self(self.0 & !(jrinx_config::PAGE_SIZE - 1))
    }

    #[cfg(feature = "pt_level_2")]
    pub fn indexes(self) -> [usize; 2] {
        [self.0 >> 22 & 0x3ff, self.0 >> 12 & 0x3ff]
    }

    #[cfg(feature = "pt_level_3")]
    pub fn indexes(self) -> [usize; 3] {
        [
            self.0 >> 30 & 0o777,
            self.0 >> 21 & 0o777,
            self.0 >> 12 & 0o777,
        ]
    }
}

pub struct PageTable {
    root: PhysAddr,
    frames: BTreeMap<VirtAddr, Arc<PhysFrame>>,
}

impl PageTable {
    pub fn new() -> Result<Self> {
        let frame = PhysFrame::alloc()?;
        let root = frame.addr();
        let mut frames = BTreeMap::new();
        frames.insert(arch::mm::phys_to_virt(root), frame);
        Ok(Self { root, frames })
    }

    fn find(&self, addr: VirtAddr) -> Result<&mut PageTableEntry> {
        let indexes = addr.indexes();
        let mut pa = self.root;
        for i in 0..indexes.len() {
            let pte = &mut pa.as_array_base::<PageTableEntry>()[indexes[i]];
            if i == indexes.len() - 1 {
                return Ok(pte);
            } else if !pte.is_valid() {
                return Err(InternalError::InvalidVirtAddr);
            }
            (pa, _) = pte.as_tuple();
        }
        Err(InternalError::InvalidVirtAddr)
    }

    fn find_or_create(&mut self, addr: VirtAddr) -> Result<&mut PageTableEntry> {
        let indexes = addr.indexes();
        let mut pa = self.root;
        for i in 0..indexes.len() {
            let pte = &mut pa.as_array_base::<PageTableEntry>()[indexes[i]];
            if i == indexes.len() - 1 {
                return Ok(pte);
            } else if !pte.is_valid() {
                let frame = PhysFrame::alloc()?;
                let addr = frame.addr();
                pte.set(addr, PagePerm::V);
                self.frames.insert(arch::mm::phys_to_virt(addr), frame);
            }
            (pa, _) = pte.as_tuple();
        }
        Err(InternalError::InvalidVirtAddr)
    }

    pub fn addr(&self) -> PhysAddr {
        self.root
    }

    pub fn translate(&self, addr: VirtAddr) -> Result<(PhysAddr, PagePerm)> {
        if let Ok((phys_frame, perm)) = self.lookup(addr) {
            Ok((
                phys_frame.addr() + (addr.as_usize() & (jrinx_config::PAGE_SIZE - 1)),
                perm,
            ))
        } else {
            Err(InternalError::InvalidVirtAddr)
        }
    }

    pub fn lookup(&self, addr: VirtAddr) -> Result<(Arc<PhysFrame>, PagePerm)> {
        let addr = addr.align_page_down();
        let pte = self.find(addr)?;
        if pte.is_valid() {
            let (_, perm) = pte.as_tuple();
            Ok((self.frames[&addr].clone(), perm))
        } else {
            Err(InternalError::InvalidVirtAddr)
        }
    }

    pub fn map(
        &mut self,
        addr: VirtAddr,
        phys_frame: Arc<PhysFrame>,
        perm: PagePerm,
    ) -> Result<()> {
        let addr = addr.align_page_down();
        let phys_addr = phys_frame.addr();
        self.frames.insert(addr, phys_frame);

        let pte = self.find_or_create(addr)?;
        pte.set(phys_addr, perm | PagePerm::V);
        Ok(())
    }

    pub fn unmap(&mut self, addr: VirtAddr) -> Result<()> {
        let addr = addr.align_page_down();
        self.frames
            .remove(&addr)
            .ok_or(InternalError::InvalidVirtAddr)?;
        let pte = self.find(addr)?;
        pte.clr();
        Ok(())
    }

    fn kernel_map(&mut self, addr: VirtAddr, perm: PagePerm) -> Result<()> {
        let addr = addr.align_page_down();
        let phys_addr = PhysAddr::new(addr.as_usize());
        let pte = self.find_or_create(addr)?;
        pte.set(phys_addr, perm | PagePerm::V);
        Ok(())
    }
}

lazy_static! {
    pub static ref KERN_PAGE_TABLE: RwLock<PageTable> = RwLock::new(PageTable::new().unwrap());
}

pub(super) fn init() {
    let mut pt = KERN_PAGE_TABLE.write();
    // Map kernel codes and data
    let mapping = [
        (
            ".text",
            jrinx_layout::_stext(),
            jrinx_layout::_etext(),
            PagePerm::G | PagePerm::X | PagePerm::R | PagePerm::V,
        ),
        (
            ".rodata",
            jrinx_layout::_srodata(),
            jrinx_layout::_erodata(),
            PagePerm::G | PagePerm::R | PagePerm::V,
        ),
        (
            ".data",
            jrinx_layout::_sdata(),
            jrinx_layout::_edata(),
            PagePerm::G | PagePerm::R | PagePerm::W | PagePerm::V,
        ),
        (
            ".bss",
            jrinx_layout::_sbss(),
            jrinx_layout::_ebss(),
            PagePerm::G | PagePerm::R | PagePerm::W | PagePerm::V,
        ),
    ];

    for (name, st, ed, perm) in mapping {
        info!(
            "mapping kernel {:7} ({} - {}) with perm {}",
            name,
            PhysAddr::new(st),
            PhysAddr::new(ed).align_page_up(),
            perm
        );
        for addr in (st..ed).step_by(jrinx_config::PAGE_SIZE) {
            pt.kernel_map(VirtAddr::new(addr), perm).unwrap();
        }
    }

    // Map physical memory regions
    for &(addr, size) in &*phys::get_init_regions() {
        let perm = PagePerm::G | PagePerm::R | PagePerm::W | PagePerm::V;
        info!(
            "mapping memory region  ({} - {}) with perm {}",
            addr,
            addr + size,
            perm
        );
        for addr in (addr.as_usize()..(addr + size).as_usize())
            .step_by(jrinx_config::PAGE_SIZE)
            .map(PhysAddr::new)
        {
            pt.kernel_map(arch::mm::phys_to_virt(addr), perm).unwrap();
        }
    }
    debug!("enable page table ({}) mapping", pt.addr());
    arch::mm::virt::enable_pt_mapping(&pt);
}
