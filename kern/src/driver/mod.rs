pub mod bootargs;
pub mod serial;

use core::{fmt::Display, mem};

use alloc::{
    borrow::ToOwned,
    string::{String, ToString},
    vec::Vec,
};
use fdt::{node::FdtNode, Fdt};

use crate::{conf, error::Result};

#[repr(C)]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum DevIdent {
    Compatible(&'static str),
    DeviceType(&'static str),
    NodePath(&'static str),
}

impl Display for DevIdent {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                DevIdent::Compatible(compat) => compat,
                DevIdent::DeviceType(devtyp) => devtyp,
                DevIdent::NodePath(path) => path,
            }
        )
    }
}

#[repr(C)]
pub struct DevReg {
    pub ident: DevIdent,
    pub probe: fn(node: &FdtNode) -> Result<()>,
}

impl DevReg {
    fn suit(&self, node: &FdtNode) -> Option<fn(node: &FdtNode) -> Result<()>> {
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

    let devs: Vec<&DevReg> = (conf::layout::_sdev()..conf::layout::_edev())
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
                .for_each(|node| do_probe(probe, path.to_owned(), &node));
        });

    // probe compatible/device-type specific devices
    dt.all_nodes().for_each(|node| {
        devs.iter()
            .filter_map(|dev| dev.suit(&node).map(|probe| (&dev.ident, probe)))
            .for_each(|(ident, probe)| do_probe(probe, ident.to_string(), &node));
    });

    if let Some(bootargs) = dt.chosen().bootargs() {
        bootargs::set(bootargs);
    }
}

fn do_probe<P>(probe: P, ident: String, node: &FdtNode)
where
    P: Fn(&FdtNode) -> Result<()>,
{
    trace!("probe {} begin", ident);
    if let Err(err) = probe(node) {
        error!("probe {} failed: {:?}", ident, err);
    } else {
        trace!("probe {} end", ident);
    }
}
