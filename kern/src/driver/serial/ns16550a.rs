use fdt::node::FdtNode;

use crate::{driver, error::Result};

fn probe(_node: &FdtNode) -> Result<()> {
    Ok(())
}

pub fn init() {
    driver::register! {
        compat("ns16550a") => probe
    };
}
