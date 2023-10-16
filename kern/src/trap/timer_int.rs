use crate::{arch::trap::Context, cpudata};

pub fn handle(_: &mut Context) {
    while let Some(tracker) =
        cpudata::with_cpu_timed_event_queue(|queue| queue.peek_outdated()).unwrap()
    {
        if let Err(err) = tracker.timeout() {
            warn!("Failed to handle timed event timeout: {:?}", err);
        }
    }
}
