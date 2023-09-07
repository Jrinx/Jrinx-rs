#[cfg(target_arch = "riscv32")]
mod riscv32;
#[cfg(target_arch = "riscv32")]
pub use riscv32::*;

#[cfg(target_arch = "riscv64")]
mod riscv64;
#[cfg(target_arch = "riscv64")]
pub use riscv64::*;

use crate::error::HaltReason;

pub mod cpu;
pub mod cpus;
pub mod layout;
pub mod mm;

#[naked]
#[no_mangle]
#[link_section = ".boot"]
unsafe extern "C" fn _start() -> ! {
    extern "C" {
        fn _estack();
    }
    core::arch::asm!(
        "la sp, {INIT_STACK_TOP}",
        "mv tp, a0",
        "call {MAIN}",
        INIT_STACK_TOP = sym _estack,
        MAIN = sym crate::main,
        options(noreturn),
    );
}

pub fn init() {
    unsafe {
        riscv::register::sstatus::set_sum();
    }
}

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
