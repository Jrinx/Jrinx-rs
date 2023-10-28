use fdt::node::FdtNode;
use jrinx_devprober_macro::devprober;
use jrinx_error::{InternalError, Result};
use spin::Once;

#[devprober(path = "/cpus")]
fn probe(node: &FdtNode) -> Result<()> {
    let timebase_freq = node
        .property("timebase-frequency")
        .ok_or(InternalError::DevProbeError)?
        .as_usize()
        .ok_or(InternalError::DevProbeError)?;

    debug!("cpus timebase-frequency: {} Hz", timebase_freq);
    TIMEBASE_FREQ
        .try_call_once::<_, ()>(|| Ok(timebase_freq))
        .unwrap();

    let nproc = node
        .children()
        .filter(|node| match node.compatible() {
            Some(compatible) => compatible.all().any(|c| c == "riscv"),
            None => false,
        })
        .count();
    debug!("cpus nproc: {}", nproc);
    NPROC.try_call_once::<_, ()>(|| Ok(nproc)).unwrap();

    Ok(())
}

static TIMEBASE_FREQ: Once<usize> = Once::new();

pub fn timebase_freq() -> Option<usize> {
    TIMEBASE_FREQ.get().copied()
}

static NPROC: Once<usize> = Once::new();

pub fn nproc() -> Option<usize> {
    NPROC.get().copied()
}
