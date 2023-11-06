use alloc::{collections::BTreeMap, sync::Arc};
use jrinx_addr::{PhysAddr, VirtAddr};
use jrinx_error::{InternalError, Result};
use jrinx_phys_frame::PhysFrame;

use crate::{
    CloneKernel, GenericPagePerm, GenericPageTable, GenericPageTableEntry, PagePerm, PageTableEntry,
};

pub struct PageTable {
    root: PhysAddr,
    frames: BTreeMap<VirtAddr, Arc<PhysFrame>>,
}

impl GenericPageTable<PagePerm, PageTableEntry> for PageTable {
    fn addr(&self) -> PhysAddr {
        self.root
    }

    fn translate(&self, addr: VirtAddr) -> jrinx_error::Result<(PhysAddr, PagePerm)> {
        if let Ok((phys_frame, perm)) = self.lookup(addr) {
            Ok((
                phys_frame.addr() + (addr.as_usize() & (jrinx_config::PAGE_SIZE - 1)),
                perm,
            ))
        } else {
            Err(InternalError::InvalidVirtAddr)
        }
    }

    fn lookup(&self, addr: VirtAddr) -> jrinx_error::Result<(Arc<PhysFrame>, PagePerm)> {
        let addr = addr.align_page_down();
        let pte = self.find(addr)?;
        if pte.valid() {
            let (_, perm) = pte.clone().into();
            Ok((self.frames[&addr].clone(), perm))
        } else {
            Err(InternalError::InvalidVirtAddr)
        }
    }

    fn map(
        &mut self,
        addr: VirtAddr,
        phys_frame: Arc<PhysFrame>,
        perm: PagePerm,
    ) -> jrinx_error::Result<()> {
        let addr = addr.align_page_down();
        let phys_addr = phys_frame.addr();
        self.frames.insert(addr, phys_frame);

        let pte = self.find_or_create(addr)?;
        pte.set(phys_addr, perm.union(PagePerm::V));
        Ok(())
    }

    fn unmap(&mut self, addr: VirtAddr) -> jrinx_error::Result<()> {
        let addr = addr.align_page_down();
        self.frames
            .remove(&addr)
            .ok_or(InternalError::InvalidVirtAddr)?;
        let pte = self.find(addr)?;
        pte.clr();
        Ok(())
    }
}

impl PageTable {
    pub fn new() -> Result<Self> {
        let frame = PhysFrame::alloc()?;
        let root = frame.addr();
        let mut frames = BTreeMap::new();
        frames.insert(root.to_virt(), frame);
        Self::clone_kernel(root.to_virt().as_array_base());
        Ok(Self { root, frames })
    }
    fn find(&self, addr: VirtAddr) -> Result<&mut PageTableEntry> {
        let indexes = addr.indexes();
        let mut pa = self.root;
        for i in 0..indexes.len() {
            let pte = &mut pa.to_virt().as_array_base::<PageTableEntry>()[indexes[i]];
            if i == indexes.len() - 1 {
                return Ok(pte);
            } else if !pte.valid() {
                return Err(InternalError::InvalidVirtAddr);
            }
            (pa, _) = pte.clone().into();
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
            } else if !pte.valid() {
                let frame = PhysFrame::alloc()?;
                let addr = frame.addr();
                pte.set(addr, PagePerm::V);
                self.frames.insert(addr.to_virt(), frame);
            }
            (pa, _) = pte.clone().into();
        }
        Err(InternalError::InvalidVirtAddr)
    }
}
