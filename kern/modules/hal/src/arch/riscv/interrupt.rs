use riscv::register::{sip, sstatus};
use sbi::HartMask;

use crate::Interrupt;

#[derive(Debug, Clone, Copy)]
pub(crate) struct InterruptImpl;

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

    fn is_timer_pending(&self) -> bool {
        sip::read().stimer()
    }

    fn clr_soft(&self) {
        unsafe {
            core::arch::asm!("csrci sip, 2");
        }
    }

    fn send_ipi(&self, cpu_ids: &[usize]) {
        let mut mask = HartMask::new(0);
        for &cpu_id in cpu_ids {
            mask = mask.with(cpu_id);
        }
        sbi::ipi::send_ipi(mask).unwrap();
    }
}
