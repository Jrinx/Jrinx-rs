use spin::RwLock;

use crate::{GenericContext, TrapReason};

static TIMER_INT_COUNTER: RwLock<u64> = RwLock::new(0);

pub(crate) fn handle(ctx: &mut impl GenericContext) {
    let TrapReason::TimerInterrupt = ctx.trap_reason() else {
        panic!("not a timer interrupt");
    };

    *TIMER_INT_COUNTER.write() += 1;

    while let Some(tracker) = jrinx_timed_event::with_current(|tq| tq.peek_outdated()) {
        if let Err(err) = tracker.timeout() {
            warn!("Failed to handle timed event timeout: {:?}", err);
        }
    }
}

pub fn count() -> u64 {
    *TIMER_INT_COUNTER.read()
}
