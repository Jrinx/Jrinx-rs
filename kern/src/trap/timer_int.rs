use crate::{arch::trap::Context, cpudata::CpuDataVisitor};

pub fn handle(_: &mut Context) {
    while let Some(tracker) = CpuDataVisitor::new()
        .timed_event_queue(|queue| queue.peek_outdated())
        .unwrap()
    {
        if let Err(err) = tracker.timeout() {
            warn!("Failed to handle timed event timeout: {:?}", err);
        }
    }
}
