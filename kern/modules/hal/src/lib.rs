#![no_std]

extern crate alloc;

mod arch;
use core::time::Duration;

pub use arch::*;

use jrinx_addr::PhysAddr;

#[macro_export]
macro_rules! hal {
    () => {
        $crate::HAL
    };
}

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

    fn set_nproc(&self, count: usize);

    fn nproc(&self) -> usize;

    fn set_timebase_freq(&self, freq: u64);

    fn timebase_freq(&self) -> u64;

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

pub trait Timer: Send + Sync {
    fn get(&self) -> Duration;

    fn set_next(&self);
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
