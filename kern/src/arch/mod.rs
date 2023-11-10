use cfg_if::cfg_if;
use fdt::Fdt;
use jrinx_addr::VirtAddr;

cfg_if! {
    if #[cfg(any(target_arch = "riscv64", target_arch = "riscv32"))] {
        mod riscv;
    } else {
        compile_error!("unsupported target_arch");
    }
}

pub struct BootInfo {
    fdt_addr: VirtAddr,
}

impl BootInfo {
    pub fn fdt(&self) -> Fdt<'_> {
        unsafe { Fdt::from_ptr(self.fdt_addr.as_usize() as *const _).unwrap() }
    }
}
