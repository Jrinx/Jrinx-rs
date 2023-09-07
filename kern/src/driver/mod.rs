pub mod bootargs;
pub mod serial;

use core::mem;

use alloc::vec::Vec;
use fdt::{node::FdtNode, Fdt};

use crate::error::Result;

#[repr(C)]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum DevIdent {
    Compatible(&'static str),
    DeviceType(&'static str),
    NodePath(&'static str),
}

#[repr(C)]
pub struct DevReg {
    pub ident: DevIdent,
    pub probe: fn(node: &FdtNode) -> Result<()>,
}

impl DevReg {
    fn suit<'a>(&self, node: &FdtNode) -> Option<fn(node: &FdtNode) -> Result<()>> {
        match self.ident {
            DevIdent::Compatible(compat) => node
                .compatible()
                .is_some_and(|n| n.all().any(|c| c == compat))
                .then_some(self.probe),
            DevIdent::DeviceType(devtyp) => node
                .property("device_type")
                .is_some_and(|n| n.as_str().is_some_and(|t| t == devtyp))
                .then_some(self.probe),
            DevIdent::NodePath(_) => None,
        }
    }
}

macro_rules! device_probe {
    (compat($compat:literal) => $probe:ident) => {
        #[used(linker)]
        #[link_section = concat!(".dev.compat.", $compat)]
        static DEV_REG: &$crate::driver::DevReg = &$crate::driver::DevReg {
            ident: $crate::driver::DevIdent::Compatible($compat),
            probe: $probe,
        };
    };

    (devtyp($devtyp:literal) => $probe:ident) => {
        #[used(linker)]
        #[link_section = concat!(".dev.devtyp.", $devtyp)]
        static DEV_REG: &$crate::driver::DevReg = &$crate::driver::DevReg {
            ident: $crate::driver::DevIdent::DeviceType($devtyp),
            probe: $probe,
        };
    };

    (path($path:literal) => $probe:ident) => {
        #[used(linker)]
        #[link_section = concat!(".dev.path.", $path)]
        static DEV_REG: &$crate::driver::DevReg = &$crate::driver::DevReg {
            ident: $crate::driver::DevIdent::NodePath($path),
            probe: $probe,
        };
    };
}
pub(crate) use device_probe;

pub(super) fn init(fdtaddr: *const u8) {
    let dt = unsafe { Fdt::from_ptr(fdtaddr) }.unwrap();

    extern "C" {
        fn _sdev();
        fn _edev();
    }

    let devs: Vec<&DevReg> = (_sdev as usize.._edev as usize)
        .step_by(mem::size_of::<&DevReg>())
        .map(|a| unsafe { *(a as *const &DevReg) })
        .collect();

    // probe path specific devices
    devs.iter()
        .filter_map(|dev| match dev.ident {
            DevIdent::NodePath(path) => Some((path, dev.probe)),
            _ => None,
        })
        .for_each(|(path, probe)| {
            dt.find_all_nodes(path)
                .for_each(|node| probe(&node).unwrap());
        });

    // probe compatible/device-type specific devices
    dt.all_nodes().for_each(|node| {
        devs.iter()
            .filter_map(|dev| dev.suit(&node))
            .for_each(|probe| {
                probe(&node).unwrap();
            })
    });

    if let Some(bootargs) = dt.chosen().bootargs() {
        bootargs::set(bootargs);
    }
}
