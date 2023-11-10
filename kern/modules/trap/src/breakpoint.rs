use spin::RwLock;

use crate::{GenericContext, TrapReason};

static BREAKPOINT_COUNTER: RwLock<usize> = RwLock::new(0);

pub(crate) fn handle(ctx: &mut impl GenericContext) {
    let TrapReason::Breakpoint { addr } = ctx.trap_reason() else {
        panic!("not a breakpoint trap");
    };

    debug!("breakpoint at {}\n{:#x?}", addr, ctx);

    let mut counter = BREAKPOINT_COUNTER.write();
    *counter += 1;

    ctx.pc_advance();
}

pub fn count() -> usize {
    *BREAKPOINT_COUNTER.read()
}
