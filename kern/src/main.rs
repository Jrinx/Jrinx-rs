#![feature(asm_const)]
#![feature(btree_cursors)]
#![feature(iter_collect_into)]
#![feature(linkage)]
#![feature(map_try_insert)]
#![feature(naked_functions)]
#![feature(panic_info_message)]
#![feature(stmt_expr_attributes)]
#![feature(used_with_arg)]
#![deny(warnings)]
#![no_std]
#![no_main]

use cfg_if::cfg_if;
use error::HaltReason;

use crate::util::{logging, random};

extern crate alloc;

#[macro_use]
extern crate log;

cfg_if! {
    if #[cfg(any(target_arch = "riscv64", target_arch = "riscv32"))] {
        #[path = "arch/riscv/mod.rs"]
        mod arch;
    } else {
        compile_error!("unsupported target_arch");
    }
}

mod conf;
mod driver;
mod error;
mod heap;
mod mm;
mod test;
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
    driver::bootargs::execute();

    info!("init done, halt");
    arch::halt(HaltReason::NormalExit);
}
