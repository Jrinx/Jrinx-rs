use fdt::node::FdtNode;
use jrinx_devprober_macro::devprober;
use jrinx_error::{InternalError, Result};

use crate::util::once_lock::OnceLock;

#[devprober(path = "/cpus")]
fn probe(node: &FdtNode) -> Result<()> {
    let timebase_freq = node
        .property("timebase-frequency")
        .ok_or(InternalError::DevProbeError)?
        .as_usize()
        .ok_or(InternalError::DevProbeError)?;

    debug!("cpus timebase-frequency: {} Hz", timebase_freq);
    TIMEBASE_FREQ.init(timebase_freq)?;

    let nproc = node
        .children()
        .filter(|node| match node.compatible() {
            Some(compatible) => compatible.all().any(|c| c == "riscv"),
            None => false,
        })
        .count();
    debug!("cpus nproc: {}", nproc);
    NPROC.init(nproc)?;

    Ok(())
}

static TIMEBASE_FREQ: OnceLock<usize> = OnceLock::new();

pub fn timebase_freq() -> Option<usize> {
    TIMEBASE_FREQ.get().copied()
}

static NPROC: OnceLock<usize> = OnceLock::new();

pub fn nproc() -> Option<usize> {
    NPROC.get().copied()
}
