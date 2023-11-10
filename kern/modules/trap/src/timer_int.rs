use crate::GenericContext;

pub(crate) fn handle(_: &mut impl GenericContext) {
    while let Some(tracker) = jrinx_timed_event::with_current(|tq| tq.peek_outdated()) {
        if let Err(err) = tracker.timeout() {
            warn!("Failed to handle timed event timeout: {:?}", err);
        }
    }
}
