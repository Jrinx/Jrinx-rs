use core::time::Duration;

use riscv::register;
use spin::Once;

use crate::Cpu;

#[derive(Debug, Clone, Copy)]
pub struct CpuImpl;

static CPU_COUNT: Once<usize> = Once::new();
static CPU_VALID_COUNT: Once<usize> = Once::new();
static CPU_TIMEBASE_FREQ: Once<u64> = Once::new();

pub(crate) static CPU: CpuImpl = CpuImpl;

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

    fn set_nproc_valid(&self, count: usize) {
        CPU_VALID_COUNT.call_once(|| count);
    }

    fn nproc_valid(&self) -> usize {
        *CPU_VALID_COUNT.get().unwrap_or(&0)
    }

    fn set_nproc(&self, count: usize) {
        CPU_COUNT.call_once(|| count);
    }

    fn nproc(&self) -> usize {
        *CPU_COUNT.get().unwrap_or(&0)
    }

    fn set_timebase_freq(&self, freq: u64) {
        CPU_TIMEBASE_FREQ.call_once(|| freq);
    }

    fn timebase_freq(&self) -> u64 {
        *CPU_TIMEBASE_FREQ.get().unwrap_or(&0)
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
