use fdt::Fdt;
use jrinx_error::{InternalError, Result};
use spin::Once;

pub(in crate::arch) fn init(fdt: &Fdt<'_>) -> Result<()> {
    let node = fdt
        .find_all_nodes("/cpus")
        .next()
        .ok_or(InternalError::DevProbeError)?;

    let timebase_freq = node
        .property("timebase-frequency")
        .ok_or(InternalError::DevProbeError)?
        .as_usize()
        .ok_or(InternalError::DevProbeError)?;

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
