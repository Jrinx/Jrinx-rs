use core::time::Duration;

use riscv::register;

use crate::Cpu;

#[derive(Debug, Clone, Copy)]
pub(crate) struct CpuImpl;

impl Cpu for CpuImpl {
    fn id(&self) -> usize {
        let id;
        unsafe {
            core::arch::asm!(
                "mv {}, tp",
                out(reg) id,
            );
        }
        id
    }

    fn get_time(&self) -> core::time::Duration {
        match self.timebase_freq() {
            freq if freq != 0 => {
                Duration::from_nanos(register::time::read64() * 1_000_000_000u64 / freq)
            }
            _ => Duration::ZERO,
        }
    }

    fn set_timer(&self, next: core::time::Duration) {
        sbi::timer::set_timer(
            (next.as_nanos() * self.timebase_freq() as u128 / 1_000_000_000u128) as u64,
        )
        .unwrap();
    }
}
