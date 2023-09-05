#[cfg(target_arch = "riscv32")]
mod riscv32;
#[cfg(target_arch = "riscv32")]
pub use riscv32::*;

#[cfg(target_arch = "riscv64")]
mod riscv64;
#[cfg(target_arch = "riscv64")]
pub use riscv64::*;

use crate::error::HaltReason;

#[cfg_attr(feature = "board_virt", path = "board/virt.rs")]
mod board;

pub mod cpu;
pub mod cpus;
pub mod layout;
pub mod mm;

pub fn init() {
    unsafe {
        riscv::register::sstatus::set_sum();
    }
    cpus::init();
    board::init();
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
