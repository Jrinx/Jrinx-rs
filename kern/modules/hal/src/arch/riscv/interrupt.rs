use riscv::register::sstatus;

use crate::Interrupt;

#[derive(Debug, Clone, Copy)]
pub struct InterruptImpl;

pub(crate) static INTERRUPT: InterruptImpl = InterruptImpl;

impl Interrupt for InterruptImpl {
    fn wait(&self) {
        unsafe {
            core::arch::asm!("wfi");
        }
    }

    fn is_enabled(&self) -> bool {
        sstatus::read().sie()
    }

    fn enable(&self) {
        unsafe {
            sstatus::set_sie();
        }
    }

    fn disable(&self) {
        unsafe {
            sstatus::clear_sie();
        }
    }
}
