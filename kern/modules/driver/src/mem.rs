use alloc::vec::Vec;
use fdt::node::FdtNode;
use jrinx_addr::{PhysAddr, VirtAddr};
use jrinx_config::PHYS_MEM_BASE;
use jrinx_devprober::devprober;
use jrinx_error::{InternalError, Result};
use jrinx_util::interval::{Bound, ExclusiveIntervals};

#[devprober(device_type = "memory")]
fn probe(node: &FdtNode) -> Result<()> {
    node.reg()
        .ok_or(InternalError::DevProbeError)?
        .filter_map(|mem_region| {
            mem_region.size.map(|size| {
                let addr = PhysAddr::new(mem_region.starting_address as usize).to_virt();
                let bound = Bound::new(addr.as_usize(), size);
                let mut intervals = ExclusiveIntervals::new([bound]);
                intervals -= Bound::new(
                    PhysAddr::new(PHYS_MEM_BASE).to_virt().as_usize(),
                    VirtAddr::new(jrinx_layout::_end()).align_page_up()
                        - PhysAddr::new(PHYS_MEM_BASE).to_virt(),
                );
                intervals
                    .into_iter()
                    .map(|bound| bound.into())
                    .map(|(addr, len)| (VirtAddr::new(addr), len))
                    .collect::<Vec<_>>()
            })
        })
        .flatten()
        .for_each(jrinx_heap::enlarge);
    Ok(())
}
