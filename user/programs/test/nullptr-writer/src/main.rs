#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[no_mangle]
extern "C" fn _start() -> ! {
    unsafe {
        core::ptr::null_mut::<u8>().write_volatile(0);
    }
    unreachable!();
}

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    unreachable!();
}
