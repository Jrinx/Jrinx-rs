use core::time::Duration;

use super::cpus;

pub fn id() -> usize {
    let id: usize;
    unsafe {
        core::arch::asm!(
            "mv {}, tp",
            out(reg) id,
        );
    }
    id
}

pub fn time() -> Duration {
    match cpus::timebase_freq() {
        Some(freq) => {
            Duration::from_nanos((riscv::register::time::read() * 1_000_000_000 / freq) as u64)
        }
        None => Duration::ZERO,
    }
}
