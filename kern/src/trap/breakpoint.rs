use spin::RwLock;

use crate::{arch::AbstractContext, trap::TrapReason};

static BREAKPOINT_COUNTER: RwLock<usize> = RwLock::new(0);

pub fn handle(ctx: &mut impl AbstractContext) {
    let TrapReason::Breakpoint { addr } = ctx.trap_reason() else {
        panic!("not a breakpoint trap");
    };

    debug!("breakpoint at {}\n{:#x?}", addr, ctx);

    let mut counter = BREAKPOINT_COUNTER.write();
    *counter += 1;

    ctx.acc_pc();
}

pub fn count() -> usize {
    *BREAKPOINT_COUNTER.read()
}
