pub mod bootargs;
pub mod random;
pub mod serial;

use alloc::{borrow::ToOwned, collections::BTreeMap, vec::Vec};
use fdt::{node::FdtNode, Fdt};
use spin::Mutex;

use crate::{error::Result, info};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum DevIdent {
    Compatible(&'static str),
    DeviceType(&'static str),
    NodePath(&'static str),
}

impl DevIdent {
    fn find_in<'a>(&self, dt: &'a Fdt) -> Vec<FdtNode<'_, 'a>> {
        match self {
            DevIdent::Compatible(compat) => dt
                .all_nodes()
                .filter(|node| {
                    node.compatible()
                        .is_some_and(|compat_list| compat_list.all().any(|c| c == *compat))
                })
                .collect::<Vec<_>>(),
            DevIdent::DeviceType(devtyp) => dt
                .all_nodes()
                .filter(|node| {
                    node.property("device_type").is_some_and(|dev_type| {
                        dev_type.as_str().map(|s| s == *devtyp).unwrap_or(false)
                    })
                })
                .collect::<Vec<_>>(),
            DevIdent::NodePath(path) => dt.find_all_nodes(path).collect::<Vec<_>>(),
        }
    }
}

pub static DRIVER_REGISTRY: Mutex<BTreeMap<DevIdent, fn(node: &FdtNode) -> Result<()>>> =
    Mutex::new(BTreeMap::new());

macro_rules! register {
    (compat($compat:literal) => $probe:ident) => {
        $crate::driver::DRIVER_REGISTRY
            .lock()
            .try_insert($crate::driver::DevIdent::Compatible($compat), $probe)
            .unwrap();
    };
    (devtyp($devtyp:literal) => $probe:ident) => {
        $crate::driver::DRIVER_REGISTRY
            .lock()
            .try_insert($crate::driver::DevIdent::DeviceType($devtyp), $probe)
            .unwrap();
    };
    (path($path:literal) => $probe:ident) => {
        $crate::driver::DRIVER_REGISTRY
            .lock()
            .try_insert($crate::driver::DevIdent::NodePath($path), $probe)
            .unwrap();
    };
}
pub(crate) use register;

pub(super) fn init(fdtaddr: *const u8) {
    info!("init drivers by flattened device tree at {:p}", fdtaddr);
    let dt = unsafe { Fdt::from_ptr(fdtaddr) }.unwrap();

    for (dev_ident, probe) in DRIVER_REGISTRY.lock().iter() {
        dev_ident.find_in(&dt).iter().for_each(|node| {
            probe(node).unwrap();
        });
    }

    if let Some(bootargs) = dt.chosen().bootargs() {
        info!("bootargs: {}", bootargs);
        bootargs::BOOTARGS.lock().replace(bootargs.to_owned());
    }

    random::init();
}
