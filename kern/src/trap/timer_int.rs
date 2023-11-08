use crate::{arch::trap::Context, time};

pub fn handle(_: &mut Context) {
    while let Some(tracker) = time::with_current(|tq| tq.peek_outdated()) {
        if let Err(err) = tracker.timeout() {
            warn!("Failed to handle timed event timeout: {:?}", err);
        }
    }
}
