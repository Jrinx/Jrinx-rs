#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[no_mangle]
extern "C" fn _start() -> ! {
    unsafe {
        core::ptr::null::<u8>().read_volatile();
    }
    unreachable!();
}

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    unreachable!();
}
