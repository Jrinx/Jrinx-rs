#![feature(allocator_api)]
#![feature(asm_const)]
#![feature(btree_cursors)]
#![feature(is_sorted)]
#![feature(iter_collect_into)]
#![feature(linkage)]
#![feature(map_try_insert)]
#![feature(naked_functions)]
#![feature(offset_of)]
#![feature(panic_info_message)]
#![feature(stmt_expr_attributes)]
#![feature(trait_alias)]
#![feature(used_with_arg)]
#![deny(warnings)]
#![no_std]
#![no_main]

use crate::{
    task::runtime,
    util::{logging, random},
};

extern crate alloc;

#[macro_use]
extern crate log;

mod arch;
mod conf;
mod cpudata;
mod driver;
mod error;
mod heap;
mod mm;
mod task;
mod test;
mod time;
mod trap;
mod util;

#[used(linker)]
#[link_section = ".stack"]
static INIT_STACK: [u8; conf::KSTACK_SIZE] = [0; conf::KSTACK_SIZE];

extern "C" fn cold_init(_: usize, fdtaddr: *const u8) -> ! {
    heap::init();
    logging::init();
    random::init();

    let arch = core::option_env!("ARCH").unwrap_or("unknown");
    let build_time = core::option_env!("BUILD_TIME").unwrap_or("unknown");
    let build_mode = core::option_env!("BUILD_MODE").unwrap_or("unknown");
    info!(
        "arch = {}, built at {} in {} mode",
        arch, build_time, build_mode,
    );

    arch::init();
    driver::init(fdtaddr);

    mm::init();
    cpudata::init();

    runtime::start();
}

async fn master_init() {
    driver::bootargs::execute().await;
}
