pub mod cpus;

use fdt::Fdt;
use jrinx_addr::PhysAddr;
use jrinx_paging::boot::BootPageTable;
use riscv::register::{sie, sstatus};

use crate::arch::BootInfo;

#[used(linker)]
#[link_section = ".stack"]
static INIT_STACK: [u8; jrinx_config::KSTACK_SIZE] = [0; jrinx_config::KSTACK_SIZE];

#[naked]
#[no_mangle]
#[link_section = ".boot"]
unsafe extern "C" fn _start() -> ! {
    extern "C" {
        static _estack: usize;
    }
    core::arch::asm!(
        "lla sp, {INIT_STACK_TOP}",
        "mv tp, a0",
        "mv a0, a1",
        "call {PRIMARY_INIT}",
        INIT_STACK_TOP = sym _estack,
        PRIMARY_INIT = sym primary_init,
        options(noreturn),
    );
}

#[naked]
unsafe extern "C" fn _sencondary_start() -> ! {
    core::arch::asm!(
        "mv tp, a0",
        "mv sp, a1",
        "call {SECONDARY_INIT}",
        SECONDARY_INIT = sym secondary_init,
        options(noreturn),
    );
}

unsafe extern "C" fn primary_init(fdtaddr: usize) -> ! {
    core::ptr::write_bytes(
        jrinx_layout::_sbss() as *mut u8,
        0,
        jrinx_layout::_ebss() - jrinx_layout::_sbss(),
    );

    BootPageTable.init();
    BootPageTable.start();
    sstatus::set_sum();
    sie::set_sext();
    sie::set_stimer();
    sie::set_ssoft();

    let boot_info = BootInfo {
        fdt_addr: PhysAddr::new(fdtaddr).to_virt(),
    };

    crate::primary_init(boot_info);
}

unsafe extern "C" fn secondary_init() -> ! {
    BootPageTable.start();
    sstatus::set_sum();
    sie::set_sext();
    sie::set_stimer();
    sie::set_ssoft();

    crate::secondary_init();
}

pub fn secondary_boot(fdt: &Fdt) {
    cpus::start(fdt);
}
