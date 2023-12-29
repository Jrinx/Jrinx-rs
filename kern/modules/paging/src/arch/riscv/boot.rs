use core::mem::size_of;

use jrinx_addr::{PhysAddr, VirtAddr};
use jrinx_config::{
    RemapMemRegion, PAGE_SIZE, REMAP_HUGE_PAGE_SIZE, REMAP_MEM_OFFSET, REMAP_MEM_REGIONS,
};
use riscv::{
    asm,
    register::satp::{self, Mode},
};

use crate::{GenericPagePerm, PagePerm, PageTableEntry};

#[repr(C, align(4096))]
struct BootPageTableInner([usize; PAGE_SIZE / size_of::<usize>()]);

static mut BOOT_PAGE_TABLE: BootPageTableInner =
    BootPageTableInner([0; PAGE_SIZE / size_of::<usize>()]);

pub struct BootPageTable;

impl BootPageTable {
    /// # Safety
    ///
    /// This function is used to initialize kernel remapping at the bootstrap entry **BEFORE** calling `BootPageTable::start`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// // system entry
    /// BootPageTable::init();
    /// BootPageTable::start();
    /// ```
    #[inline(always)]
    pub unsafe fn init(&self) {
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

    /// # Safety
    ///
    /// This function is used to enable kernel remapping at the bootstrap entry **AFTER** `BootPageTable::init`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// // system entry
    /// BootPageTable::init();
    /// BootPageTable::start();
    /// ```
    #[inline(always)]
    pub unsafe fn start(&self) {
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

    pub fn clone_into(dst: &mut [usize]) {
        const HALF: usize = PAGE_SIZE / size_of::<usize>() / 2;
        dst[HALF..].copy_from_slice(&unsafe { BOOT_PAGE_TABLE.0 }[HALF..]);
    }
}
