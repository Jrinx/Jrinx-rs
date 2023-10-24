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
        Some(freq) => Duration::from_nanos(
            riscv::register::time::read() as u64 * 1_000_000_000u64 / freq as u64,
        ),
        None => Duration::ZERO,
    }
}

pub fn set_timer(next: Duration) {
    sbi::timer::set_timer(
        (next.as_nanos() * cpus::timebase_freq().unwrap() as u128 / 1_000_000_000u128) as u64,
    )
    .unwrap();
}
