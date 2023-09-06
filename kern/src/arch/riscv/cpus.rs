use fdt::node::FdtNode;

use crate::{
    driver::device_probe,
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

device_probe! {
    path("/cpus") => probe
}
