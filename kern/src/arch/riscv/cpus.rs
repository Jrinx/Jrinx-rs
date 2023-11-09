use fdt::Fdt;
use jrinx_error::{InternalError, Result};
use jrinx_hal::{Cpu, Hal};

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

    hal!().cpu().set_timebase_freq(timebase_freq as u64);

    let nproc = node
        .children()
        .filter(|node| match node.compatible() {
            Some(compatible) => compatible.all().any(|c| c == "riscv"),
            None => false,
        })
        .count();

    hal!().cpu().set_nproc(nproc);

    Ok(())
}
