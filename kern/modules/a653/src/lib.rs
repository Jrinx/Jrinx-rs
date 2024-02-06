#![no_std]
#![feature(allocator_api)]

extern crate alloc;

#[macro_use]
extern crate log;

#[macro_use]
extern crate jrinx_hal;

pub mod partition;
pub mod process;

#[derive(Debug, Clone, Copy)]
pub enum A653Entry {
    User(usize),
    Kern(jrinx_apex::ApexSystemAddress),
}
