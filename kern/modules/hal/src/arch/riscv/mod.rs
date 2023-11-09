pub mod cache;
pub mod cpu;
pub mod earlycon;
pub mod interrupt;
pub mod vm;

use crate::{Hal, HaltReason};

#[derive(Debug, Clone, Copy)]
pub struct HalImpl;

pub static HAL: HalImpl = HalImpl;

impl Hal for HalImpl {
    fn breakpoint(&self) {
        unsafe {
            riscv::asm::ebreak();
        }
    }

    fn cpu(&self) -> impl crate::Cpu {
        cpu::CPU
    }

    fn earlycon(&self) -> impl crate::Earlycon {
        earlycon::EARLYCON
    }

    fn halt(&self, reason: crate::HaltReason) -> ! {
        let _ = sbi::system_reset::system_reset(
            sbi::system_reset::ResetType::WarmReboot,
            match reason {
                HaltReason::NormalExit => sbi::system_reset::ResetReason::NoReason,
                _ => sbi::system_reset::ResetReason::SystemFailure,
            },
        );
        unreachable!();
    }

    fn cache(&self) -> impl crate::Cache {
        cache::CACHE
    }

    fn interrupt(&self) -> impl crate::Interrupt {
        interrupt::INTERRUPT
    }

    fn vm(&self) -> impl crate::Vm {
        vm::VM
    }
}
