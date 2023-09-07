#![feature(asm_const)]
#![feature(btree_cursors)]
#![feature(iter_collect_into)]
#![feature(linkage)]
#![feature(map_try_insert)]
#![feature(naked_functions)]
#![feature(panic_info_message)]
#![feature(stmt_expr_attributes)]
#![feature(used_with_arg)]
#![no_std]
#![no_main]

use error::HaltReason;

use crate::util::logging;

extern crate alloc;

#[macro_use]
extern crate log;

#[cfg_attr(
    any(target_arch = "riscv64", target_arch = "riscv32"),
    path = "arch/riscv/mod.rs"
)]
mod arch;

mod conf;
mod driver;
mod error;
mod heap;
mod mm;
mod test;
mod util;

fn main(_: usize, fdtaddr: *const u8) -> ! {
    logging::init();

    let build_time = core::option_env!("BUILD_TIME").unwrap_or("unknown");
    let build_mode = core::option_env!("BUILD_MODE").unwrap_or("unknown");
    info!("build at {} in {} mode", build_time, build_mode);

    heap::init();
    arch::init();
    driver::init(fdtaddr);

    mm::init();
    driver::bootargs::execute();

    info!("init done, halt");
    arch::halt(HaltReason::NormalExit);
}
