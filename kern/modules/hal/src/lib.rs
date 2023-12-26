#![no_std]

extern crate alloc;

mod arch;
use core::time::Duration;

use alloc::vec::Vec;
pub use arch::*;

use jrinx_addr::PhysAddr;
use spin::Once;

#[macro_export]
macro_rules! hal {
    () => {
        $crate::HalImpl
    };
}

static CPU_COUNT: Once<usize> = Once::new();
static CPU_VALID_COUNT: Once<usize> = Once::new();
static CPU_TIMEBASE_FREQ: Once<u64> = Once::new();

pub trait Hal: Send + Sync {
    fn breakpoint(&self);

    fn cpu(&self) -> impl Cpu;

    fn earlycon(&self) -> impl Earlycon;

    fn halt(&self, reason: HaltReason) -> !;

    fn cache(&self) -> impl Cache;

    fn interrupt(&self) -> impl Interrupt;

    fn vm(&self) -> impl Vm;
}

pub trait Cpu: Send + Sync {
    fn id(&self) -> usize;

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

    fn get_time(&self) -> Duration;

    fn set_timer(&self, next: Duration);
}

pub trait Earlycon: Send + Sync {
    fn putc(&self, c: u8);

    fn getc(&self) -> Option<u8>;
}

pub trait Cache: Send + Sync {
    fn sync_all(&self);
}

pub trait Interrupt: Send + Sync {
    fn wait(&self);

    fn is_enabled(&self) -> bool;

    fn enable(&self);

    fn disable(&self);

    fn with_saved_on<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let enabled = self.is_enabled();
        self.enable();
        let ret = f();
        if !enabled {
            self.disable();
        }
        ret
    }

    fn with_saved_off<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let enabled = self.is_enabled();
        self.disable();
        let ret = f();
        if enabled {
            self.enable();
        }
        ret
    }

    fn is_timer_pending(&self) -> bool;

    fn clr_soft(&self);

    fn send_ipi(&self, cpu_ids: &[usize]);

    fn broadcast_ipi(&self) {
        self.send_ipi(
            (0..hal!().cpu().nproc())
                .filter(|&id| id != hal!().cpu().id())
                .collect::<Vec<_>>()
                .into_boxed_slice()
                .as_ref(),
        );
    }
}

pub trait Vm: Send + Sync {
    fn enable(&self, page_table: PhysAddr);

    fn disable(&self);

    fn sync_all(&self);
}

pub enum HaltReason {
    NormalExit,
    SysFailure,
}
