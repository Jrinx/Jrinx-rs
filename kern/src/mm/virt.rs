use alloc::{collections::BTreeMap, sync::Arc};
use jrinx_addr::{PhysAddr, VirtAddr};
use jrinx_error::{InternalError, Result};
use spin::{Lazy, RwLock};

use crate::arch::{
    self,
    mm::virt::{PagePerm, PageTableEntry},
    vm_enable,
};

use super::phys::PhysFrame;

pub struct PageTable {
    root: PhysAddr,
    frames: BTreeMap<VirtAddr, Arc<PhysFrame>>,
}

pub static KERN_PAGE_TABLE: Lazy<RwLock<PageTable>> =
    Lazy::new(|| RwLock::new(PageTable::new().unwrap()));

impl PageTable {
    pub fn new() -> Result<Self> {
        let frame = PhysFrame::alloc()?;
        let root = frame.addr();
        let mut frames = BTreeMap::new();
        frames.insert(root.to_virt(), frame);
        arch::vm_clone_kernel(root.to_virt().as_array_base());
        Ok(Self { root, frames })
    }

    fn find(&self, addr: VirtAddr) -> Result<&mut PageTableEntry> {
        let indexes = addr.indexes();
        let mut pa = self.root;
        for i in 0..indexes.len() {
            let pte = &mut pa.to_virt().as_array_base::<PageTableEntry>()[indexes[i]];
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
            let pte = &mut pa.to_virt().as_array_base::<PageTableEntry>()[indexes[i]];
            if i == indexes.len() - 1 {
                return Ok(pte);
            } else if !pte.is_valid() {
                let frame = PhysFrame::alloc()?;
                let addr = frame.addr();
                pte.set(addr, PagePerm::V);
                self.frames.insert(addr.to_virt(), frame);
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
}

pub(super) fn init() {
    let addr = KERN_PAGE_TABLE.read().addr();
    vm_enable(addr);
}
