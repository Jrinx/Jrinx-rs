use fdt::node::FdtNode;
use jrinx_error::Result;

use crate::driver::device_probe;

fn probe(_node: &FdtNode) -> Result<()> {
    Ok(())
}

device_probe! {
    compat("ns16550a") => probe
}
