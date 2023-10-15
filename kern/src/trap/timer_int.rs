use crate::{arch::trap::Context, cpudata};

pub fn handle(_: &mut Context) {
    let timed_event_id =
        cpudata::with_cpu_timed_event_queue(|queue| queue.peek().unwrap()).unwrap();

    cpudata::with_cpu_timed_event_queue(|queue| queue.timeout(timed_event_id).unwrap()).unwrap();
}
