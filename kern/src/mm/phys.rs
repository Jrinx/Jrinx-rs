use alloc::{alloc::Global, sync::Arc, vec, vec::Vec};
use fdt::node::FdtNode;
use jrinx_addr::{PhysAddr, VirtAddr};
use jrinx_devprober_macro::devprober;
use jrinx_error::{InternalError, Result};
use spin::{Mutex, MutexGuard};

use crate::{arch, heap};

use core::{
    alloc::{Allocator, Layout},
    ptr::NonNull,
};

static INIT_MEM_REGIONS: Mutex<Vec<(VirtAddr, usize)>> = Mutex::new(Vec::new());

#[devprober(device_type = "memory")]
fn probe(node: &FdtNode) -> Result<()> {
    let mut init_mem_regions = INIT_MEM_REGIONS.lock();
    node.reg()
        .ok_or(InternalError::DevProbeError)?
        .filter_map(|mem_region| {
            let addr = arch::mm::phys_to_virt(PhysAddr::new(mem_region.starting_address as usize));
            if let Some(size) = mem_region.size {
                trace!("probed physical memory region: {} - {}", addr, addr + size);
                Some(
                    arch::mm::get_protected_mem_regions()
                        .iter()
                        .filter_map(|&(protected_addr, protected_size)| {
                            if protected_addr >= addr
                                && protected_addr + protected_size <= addr + size
                            {
                                if protected_addr == addr && protected_size == size {
                                    None
                                } else if protected_addr == addr {
                                    Some(vec![(
                                        protected_addr + protected_size,
                                        size - protected_size,
                                    )])
                                } else if protected_addr + protected_size == addr + size {
                                    Some(vec![(addr, protected_addr - addr)])
                                } else {
                                    Some(vec![
                                        (addr, protected_addr - addr),
                                        (protected_addr + protected_size, size - protected_size),
                                    ])
                                }
                            } else {
                                Some(vec![(addr, size)])
                            }
                        })
                        .collect::<Vec<_>>(),
                )
            } else {
                None
            }
        })
        .flatten()
        .flatten()
        .collect_into(&mut *init_mem_regions);

    init_mem_regions
        .iter()
        .for_each(|&region| heap::enlarge(region));

    Ok(())
}

pub(super) fn get_init_regions() -> MutexGuard<'static, Vec<(VirtAddr, usize)>> {
    INIT_MEM_REGIONS.lock()
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysFrame {
    addr: PhysAddr,
}

#[repr(C, align(4096))]
struct PhysFrameMemory([u8; jrinx_config::PAGE_SIZE]);

const PHYS_FRAME_MEMORY_LAYOUT: Layout = Layout::new::<PhysFrameMemory>();

impl Drop for PhysFrame {
    fn drop(&mut self) {
        unsafe {
            Global.deallocate(
                NonNull::new(arch::mm::phys_to_virt(self.addr()).as_usize() as *mut u8).unwrap(),
                PHYS_FRAME_MEMORY_LAYOUT,
            );
        }
    }
}

impl PhysFrame {
    pub fn alloc() -> Result<Arc<Self>> {
        let addr: NonNull<u8> = Global
            .allocate_zeroed(PHYS_FRAME_MEMORY_LAYOUT)
            .map_err(|_| InternalError::NotEnoughMem)?
            .cast();
        let addr = addr.as_ptr() as usize;
        let addr = arch::mm::virt_to_phys(VirtAddr::new(addr));
        Ok(Arc::new(Self { addr }))
    }

    pub fn addr(&self) -> PhysAddr {
        self.addr
    }
}
