use core::panic::PanicInfo;

use crate::{arch, cpudata, error::HaltReason, task::sched};

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
    if cpudata::get_current_task().is_none() {
        arch::halt(HaltReason::SysFailure);
    } else {
        sched::global_destroy();
    }
}
