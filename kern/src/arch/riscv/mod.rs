pub mod boot;
pub mod cpu;
pub mod cpus;
pub mod earlycon;
pub mod mm;
pub mod task;
pub mod trap;

use jrinx_addr::{PhysAddr, VirtAddr};
use jrinx_config::PAGE_SIZE;
use jrinx_error::HaltReason;
use riscv::register::satp::{self, Mode};

use boot::paging::BootPageTable;

pub fn halt(reason: HaltReason) -> ! {
    let _ = sbi::system_reset::system_reset(
        sbi::system_reset::ResetType::WarmReboot,
        match reason {
            HaltReason::NormalExit => sbi::system_reset::ResetReason::NoReason,
            _ => sbi::system_reset::ResetReason::SystemFailure,
        },
    );
    unreachable!();
}

pub fn wait_for_interrupt() {
    unsafe {
        riscv::asm::wfi();
    }
}

pub fn inst_barrier() {
    unsafe {
        core::arch::asm!("fence.i");
    }
}

pub fn vm_enable(page_table: PhysAddr) {
    #[cfg(target_arch = "riscv32")]
    unsafe {
        satp::set(Mode::Sv32, 0, page_table.as_usize() / PAGE_SIZE);
    }

    #[cfg(target_arch = "riscv64")]
    unsafe {
        satp::set(Mode::Sv39, 0, page_table.as_usize() / PAGE_SIZE);
    }

    vm_barrier_all();
}

pub fn vm_clone_kernel(page: &mut [usize]) {
    BootPageTable::clone_into(page);
}

pub fn vm_barrier_all() {
    unsafe {
        core::arch::asm!("sfence.vma x0, x0");
    }
}

pub fn vm_barrier(addr: VirtAddr) {
    unsafe {
        core::arch::asm!("sfence.vma {}, x0", in(reg) addr.as_usize());
    }
}

pub fn breakpoint() {
    unsafe {
        riscv::asm::ebreak();
    }
}

pub fn int_is_enabled() -> bool {
    riscv::register::sstatus::read().sie()
}

pub fn int_enable() {
    unsafe {
        riscv::register::sstatus::set_sie();
    }
}

pub fn int_disable() {
    unsafe {
        riscv::register::sstatus::clear_sie();
    }
}
