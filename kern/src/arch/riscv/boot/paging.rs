use core::mem::size_of;

use jrinx_addr::{PhysAddr, VirtAddr};
use jrinx_config::{
    RemapMemRegion, PAGE_SIZE, REMAP_HUGE_PAGE_SIZE, REMAP_MEM_OFFSET, REMAP_MEM_REGIONS,
};
use riscv::{
    asm,
    register::satp::{self, Mode},
};

use crate::arch::mm::virt::{PagePerm, PageTableEntry};

#[repr(C, align(4096))]
pub(in crate::arch) struct BootPageTable([usize; PAGE_SIZE / size_of::<usize>()]);

pub(in crate::arch) static mut BOOT_PAGE_TABLE: BootPageTable =
    BootPageTable([0; PAGE_SIZE / size_of::<usize>()]);

impl BootPageTable {
    #[inline(always)]
    pub(in crate::arch) unsafe fn init() {
        for &RemapMemRegion {
            virt_addr,
            phys_addr,
            len,
        } in REMAP_MEM_REGIONS
        {
            for i in 0..(len / REMAP_HUGE_PAGE_SIZE) {
                let vaddr = VirtAddr::new(virt_addr + i * REMAP_HUGE_PAGE_SIZE);
                let paddr = PhysAddr::new(phys_addr + i * REMAP_HUGE_PAGE_SIZE);
                BOOT_PAGE_TABLE.0[vaddr.indexes()[0]] = PageTableEntry::new(
                    paddr,
                    PagePerm::G | PagePerm::X | PagePerm::W | PagePerm::R | PagePerm::V,
                )
                .into();
            }
        }
    }

    #[inline(always)]
    pub(in crate::arch) unsafe fn start() {
        let pt_ppn: usize = BOOT_PAGE_TABLE.0.as_ptr() as usize / PAGE_SIZE;

        #[cfg(target_arch = "riscv32")]
        satp::set(Mode::Sv32, 0, pt_ppn);

        #[cfg(target_arch = "riscv64")]
        satp::set(Mode::Sv39, 0, pt_ppn);

        asm::sfence_vma_all();

        Self::jump_high_addr();
    }

    #[naked]
    unsafe extern "C" fn jump_high_addr() {
        core::arch::asm!("li t0, {OFFSET}", "add sp, sp, t0", "add ra, ra, t0", "ret", OFFSET = const REMAP_MEM_OFFSET, options(noreturn),);
    }

    pub(in crate::arch) fn clone_into(dst: &mut [usize]) {
        dst.clone_from_slice(&unsafe { BOOT_PAGE_TABLE.0 });
    }
}
