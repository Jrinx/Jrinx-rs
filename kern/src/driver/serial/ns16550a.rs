use fdt::node::FdtNode;

use crate::{driver::device_probe, error::Result};

fn probe(_node: &FdtNode) -> Result<()> {
    Ok(())
}

device_probe! {
    compat("ns16550a") => probe
}
