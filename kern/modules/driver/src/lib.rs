#![no_std]
#![feature(used_with_arg)]

extern crate alloc;

#[macro_use]
extern crate log;

mod mem;
mod serial;

use fdt::Fdt;

pub fn probe_all(fdt: &Fdt<'_>) {
    info!("probing all devices");
    jrinx_devprober::probe_all_device(fdt).unwrap();
}
