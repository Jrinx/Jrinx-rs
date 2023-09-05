use fdt::node::FdtNode;

use crate::{
    driver,
    error::{InternalError, Result},
    info,
};

fn probe(node: &FdtNode) -> Result<()> {
    let timebase_freq = node
        .property("timebase-frequency")
        .ok_or(InternalError::DevProbeError)?
        .as_usize()
        .ok_or(InternalError::DevProbeError)?;
    info!("cpus timebase-frequency: {} Hz", timebase_freq);
    Ok(())
}

pub fn init() {
    driver::register! {
        path("/cpus") => probe
    };
}
