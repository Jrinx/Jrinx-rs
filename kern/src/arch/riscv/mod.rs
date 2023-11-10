mod cpus;

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
        "call {INIT}",
        INIT_STACK_TOP = sym _estack,
        INIT = sym init,
        options(noreturn),
    );
}

unsafe extern "C" fn init(fdtaddr: usize) -> ! {
    (jrinx_layout::_sbss()..jrinx_layout::_ebss()).for_each(|x| {
        core::ptr::write_volatile(x as *mut u8, 0);
    });

    BootPageTable::init();
    BootPageTable::start();
    sstatus::set_sum();
    sie::set_sext();
    sie::set_stimer();
    sie::set_ssoft();

    let boot_info = BootInfo {
        fdt_addr: PhysAddr::new(fdtaddr).to_virt(),
    };

    cpus::init(&boot_info.fdt());
    crate::cold_init(boot_info);
}
