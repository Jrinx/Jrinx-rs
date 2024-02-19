use jrinx_hal::{hal, Hal, Interrupt};
use spin::RwLock;

use crate::{GenericContext, TrapReason};

static SOFT_INT_COUNTER: RwLock<u64> = RwLock::new(0);

pub(crate) fn handle(ctx: &mut impl GenericContext) {
    let TrapReason::SoftwareInterrupt = ctx.trap_reason() else {
        panic!("not a software interrupt");
    };

    *SOFT_INT_COUNTER.write() += 1;

    while let Some(event) = jrinx_intercpu_event::with_current(|iq| iq.pop()) {
        event.fire();
    }

    hal!().interrupt().clr_soft();
}

pub fn count() -> u64 {
    *SOFT_INT_COUNTER.read()
}
