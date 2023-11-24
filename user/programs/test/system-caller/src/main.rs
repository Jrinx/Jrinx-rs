#![feature(naked_functions)]
#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[naked]
#[no_mangle]
unsafe extern "C" fn _start() -> ! {
    #[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
    core::arch::asm!("li a7, 0xC0DE", "ecall", options(noreturn));
}

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    unreachable!();
}
