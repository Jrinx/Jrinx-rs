pub mod bootargs;
pub mod serial;

use fdt::Fdt;

pub(super) fn init(fdtaddr: *const u8) {
    let fdt = unsafe { Fdt::from_ptr(fdtaddr) }.unwrap();

    jrinx_devprober::probe_all_device(&fdt).unwrap();

    if let Some(bootargs) = fdt.chosen().bootargs() {
        bootargs::set(bootargs);
    }
}
