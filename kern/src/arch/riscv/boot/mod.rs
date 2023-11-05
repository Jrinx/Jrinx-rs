use jrinx_addr::PhysAddr;
use riscv::register::{sie, sstatus};

use crate::arch::{trap, BootInfo};

use self::paging::BootPageTable;

use super::cpus;

pub(in crate::arch) mod paging;

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
        "la sp, {INIT_STACK_TOP}",
        "mv tp, a0",
        "mv a0, a1",
        "call {INIT}",
        INIT_STACK_TOP = sym _estack,
        INIT = sym init,
        options(noreturn),
    );
}

unsafe extern "C" fn init(fdtaddr: usize) -> ! {
    BootPageTable::init();
    BootPageTable::start();
    sstatus::set_sum();
    sie::set_sext();
    sie::set_stimer();
    sie::set_ssoft();
    trap::init();

    let boot_info = BootInfo {
        fdt_addr: PhysAddr::new(fdtaddr).to_virt(),
    };

    cpus::init(&boot_info.fdt()).unwrap();
    crate::cold_init(boot_info);
}