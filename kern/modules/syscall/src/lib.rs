#![no_std]

mod all;
mod partition;
mod process;

extern crate alloc;

#[macro_use]
extern crate jrinx_hal;

pub use all::{handle, pinned_handle};
