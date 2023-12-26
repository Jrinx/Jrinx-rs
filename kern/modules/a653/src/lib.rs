#![no_std]
#![feature(allocator_api)]

extern crate alloc;

#[macro_use]
extern crate jrinx_hal;

pub mod partition;
pub mod process;

pub use a653rs::bindings;
