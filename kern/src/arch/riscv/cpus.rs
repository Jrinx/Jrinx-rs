use fdt::node::FdtNode;

use crate::{
    driver::device_probe,
    error::{InternalError, Result},
};

fn probe(node: &FdtNode) -> Result<()> {
    let timebase_freq = node
        .property("timebase-frequency")
        .ok_or(InternalError::DevProbeError)?
        .as_usize()
        .ok_or(InternalError::DevProbeError)?;
    unsafe {
        TIMEBASE_FREQ = timebase_freq;
    }
    info!("cpus timebase-frequency: {} Hz", timebase_freq);
    Ok(())
}

device_probe! {
    path("/cpus") => probe
}

static mut TIMEBASE_FREQ: usize = 0;

pub fn timebase_freq() -> Option<usize> {
    match unsafe { TIMEBASE_FREQ } {
        0 => None,
        freq => Some(freq),
    }
}
