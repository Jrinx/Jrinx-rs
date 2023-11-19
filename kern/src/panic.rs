use core::panic::PanicInfo;

use jrinx_hal::{Hal, HaltReason};

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if let Some(location) = info.location() {
        error!(
            "panicked at {}:{} {}",
            location.file(),
            location.line(),
            info.message().unwrap()
        );
    } else {
        error!("panicked: {}", info.message().unwrap());
    }
    hal!().halt(HaltReason::SysFailure);
}
