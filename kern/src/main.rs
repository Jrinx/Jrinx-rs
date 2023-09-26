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
#![feature(used_with_arg)]
#![deny(warnings)]
#![no_std]
#![no_main]

use crate::{
    task::{
        sched::{self, Scheduler},
        Task,
    },
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
mod trap;
mod util;

#[used(linker)]
#[link_section = ".stack"]
static INIT_STACK: [u8; conf::KSTACK_SIZE] = [0; conf::KSTACK_SIZE];

fn cold_init(_: usize, fdtaddr: *const u8) -> ! {
    logging::init();
    random::init();

    let arch = core::option_env!("ARCH").unwrap_or("unknown");
    let build_time = core::option_env!("BUILD_TIME").unwrap_or("unknown");
    let build_mode = core::option_env!("BUILD_MODE").unwrap_or("unknown");
    info!(
        "target-arch is {}, build at {} in {} mode",
        arch, build_time, build_mode
    );

    heap::init();
    arch::init();
    driver::init(fdtaddr);

    mm::init();
    cpudata::init();

    sched::with_global_scheduler(|scheduler| {
        scheduler.insert(Task::create("init", 0, root_task_init as usize, 0).unwrap());
    });
    sched::global_sched_start();
}

fn root_task_init(_: usize) {
    info!(
        "root task '{}' started",
        cpudata::get_current_task().unwrap().get_name()
    );
    driver::bootargs::execute();
}
