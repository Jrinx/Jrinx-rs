#![no_std]
#![feature(allocator_api)]

extern crate alloc;

#[macro_use]
extern crate log;

#[macro_use]
extern crate jrinx_hal;

pub mod helper;
pub mod partition;
pub mod process;

pub use a653rs::bindings;

#[derive(Debug, Clone, Copy)]
pub enum A653Entry {
    User(usize),
    Kern(a653rs::bindings::SystemAddress),
}
