use riscv::register::satp::{self, Mode};

use crate::Vm;

#[derive(Debug, Clone, Copy)]
pub struct VmImpl;

pub(crate) static VM: VmImpl = VmImpl;

impl Vm for VmImpl {
    fn enable(&self, page_table: jrinx_addr::PhysAddr) {
        #[cfg(target_arch = "riscv32")]
        unsafe {
            satp::set(Mode::Sv32, 0, page_table.as_usize() >> 12);
        }

        #[cfg(target_arch = "riscv64")]
        unsafe {
            satp::set(Mode::Sv39, 0, page_table.as_usize() >> 12);
        }
    }

    fn disable(&self) {
        unsafe {
            satp::set(Mode::Bare, 0, 0);
        }
    }

    fn sync_all(&self) {
        unsafe {
            riscv::asm::sfence_vma_all();
        }
    }
}
