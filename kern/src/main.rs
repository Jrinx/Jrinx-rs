#![feature(asm_const)]
#![feature(const_binary_heap_constructor)]
#![feature(iter_collect_into)]
#![feature(map_try_insert)]
#![feature(naked_functions)]
#![feature(offset_of)]
#![feature(panic_info_message)]
#![feature(used_with_arg)]
#![deny(warnings)]
#![no_std]
#![no_main]

use arch::BootInfo;

use crate::{task::runtime, util::logging};

use jrinx_hal::{Cpu, Hal};

extern crate alloc;
#[macro_use]
extern crate log;

extern crate jrinx_driver as _;
#[macro_use]
extern crate jrinx_hal;

mod arch;
mod bootargs;
mod task;
mod test;
mod time;
mod trap;
mod util;
mod vmm;

fn cold_init(boot_info: BootInfo) -> ! {
    jrinx_heap::init();
    logging::init();

    jrinx_percpu::init(hal!().cpu().nproc());
    jrinx_percpu::set_local_pointer(hal!().cpu().id());

    let arch = core::option_env!("ARCH").unwrap_or("unknown");
    let build_time = core::option_env!("BUILD_TIME").unwrap_or("unknown");
    let build_mode = core::option_env!("BUILD_MODE").unwrap_or("unknown");
    info!(
        "arch = {}, built at {} in {} mode",
        arch, build_time, build_mode,
    );

    jrinx_driver::probe_all(&boot_info.fdt());

    if let Some(bootargs) = boot_info.fdt().chosen().bootargs() {
        bootargs::set(bootargs);
    }

    vmm::init();
    runtime::init(master_init());
    runtime::start();
}

async fn master_init() {
    bootargs::execute().await;
}
