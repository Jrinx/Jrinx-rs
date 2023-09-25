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
    debug!("cpus timebase-frequency: {} Hz", timebase_freq);

    let nproc = node
        .children()
        .filter_map(|node| {
            debug!("node: {}", node.name);
            match node.compatible() {
                Some(compatible) => compatible.all().any(|c| c == "riscv").then_some(node),
                None => None,
            }
        })
        .count();
    debug!("cpus nproc: {}", nproc);

    unsafe {
        NPROC = nproc;
    }

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

static mut NPROC: usize = 0;

pub fn nproc() -> Option<usize> {
    match unsafe { NPROC } {
        0 => None,
        nproc => Some(nproc),
    }
}
