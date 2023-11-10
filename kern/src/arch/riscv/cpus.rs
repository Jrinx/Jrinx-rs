use fdt::Fdt;
use jrinx_hal::{Cpu, Hal};

pub(in crate::arch) fn init(fdt: &Fdt<'_>) {
    let node = fdt.find_all_nodes("/cpus").next().unwrap();

    let timebase_freq = node
        .property("timebase-frequency")
        .unwrap()
        .as_usize()
        .unwrap();

    hal!().cpu().set_timebase_freq(timebase_freq as u64);

    let nproc = node
        .children()
        .filter(|node| match node.compatible() {
            Some(compatible) => compatible.all().any(|c| c == "riscv"),
            None => false,
        })
        .count();

    hal!().cpu().set_nproc(nproc);
}
