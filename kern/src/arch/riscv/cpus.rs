use core::alloc::{Allocator, Layout};

use alloc::alloc::Global;
use fdt::{node::FdtNode, Fdt};
use jrinx_addr::VirtAddr;
use jrinx_hal::{Cpu, Hal};
use sbi::base::{probe_extension, ExtensionAvailability};

fn is_cpu(node: &FdtNode) -> bool {
    node.name == "cpu" || node.name.starts_with("cpu@")
}

fn is_valid_cpu(node: &FdtNode) -> bool {
    is_cpu(node)
        && !node
            .property("status")
            .is_some_and(|prop| prop.as_str().is_some_and(|status| status != "okay"))
}

pub fn init(fdt: &Fdt<'_>) {
    let node = fdt.find_all_nodes("/cpus").next().unwrap();

    hal!().cpu().set_timebase_freq(
        node.property("timebase-frequency")
            .unwrap()
            .as_usize()
            .unwrap() as u64,
    );

    hal!()
        .cpu()
        .set_nproc(node.children().filter(is_cpu).count());

    hal!()
        .cpu()
        .set_nproc_valid(node.children().filter(is_valid_cpu).count());
}

pub(in crate::arch) fn start(fdt: &Fdt<'_>) {
    let node = fdt.find_all_nodes("/cpus").next().unwrap();

    if let ExtensionAvailability::Available(_) = probe_extension(sbi::hsm::EXTENSION_ID) {
        for cpu in node.children().filter(is_valid_cpu) {
            let id = if cpu.name == "cpu" {
                0
            } else if cpu.name.starts_with("cpu@") {
                cpu.name[4..].parse::<usize>().unwrap()
            } else {
                continue;
            };
            if id == hal!().cpu().id() {
                continue;
            }
            let entry = VirtAddr::new(super::_sencondary_start as usize);
            let stack_top = VirtAddr::new(
                Global
                    .allocate(
                        Layout::from_size_align(jrinx_config::KSTACK_SIZE, jrinx_config::PAGE_SIZE)
                            .unwrap(),
                    )
                    .unwrap()
                    .as_ptr()
                    .cast::<u8>() as usize,
            ) + jrinx_config::KSTACK_SIZE;
            sbi::hsm::hart_start(
                id,
                entry.to_phys().as_usize(),
                stack_top.to_phys().as_usize(),
            )
            .unwrap();
        }
    }
}
