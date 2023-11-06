#![no_std]

extern crate alloc;

mod arch;
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

    fn earlycon(&self) -> impl Earlycon;

    fn halt(&self, reason: HaltReason) -> !;

    fn cache(&self) -> impl Cache;

    fn interrupt(&self) -> impl Interrupt;

    fn vm(&self) -> impl Vm;
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
