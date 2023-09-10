use spin::RwLock;

static BREAKPOINT_COUNTER: RwLock<usize> = RwLock::new(0);

pub fn do_breakpoint(addr: usize) {
    debug!("breakpoint at {:#x}", addr);
    let mut counter = BREAKPOINT_COUNTER.write();
    *counter += 1;
}

pub fn get_breakpoint_count() -> usize {
    *BREAKPOINT_COUNTER.read()
}
