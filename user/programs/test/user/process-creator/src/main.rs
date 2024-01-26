#![no_std]
#![no_main]
#![feature(panic_info_message)]

#[macro_use]
extern crate log;

use core::panic::PanicInfo;

use jrlib_a653::{bindings::*, helper::convert_str_to_name, prelude::*};
use jrlib_sys::sys_debug_halt;

extern "C" fn entry() {
    info!("Hello World!");
    loop {}
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    jrlib_logging::init();

    let proc_1 = Process::create_process(&ApexProcessAttribute {
        period: INFINITE_TIME_VALUE,
        time_capacity: INFINITE_TIME_VALUE,
        entry_point: entry,
        stack_size: 4 * 4096,
        base_priority: Priority::default(),
        deadline: Deadline::Soft,
        name: convert_str_to_name("proc_1").unwrap(),
    })
    .unwrap();

    let proc_2 = Process::create_process(&ApexProcessAttribute {
        period: INFINITE_TIME_VALUE,
        time_capacity: INFINITE_TIME_VALUE,
        entry_point: entry,
        stack_size: 4 * 4096,
        base_priority: Priority::default(),
        deadline: Deadline::Soft,
        name: convert_str_to_name("proc_2").unwrap(),
    })
    .unwrap();

    Process::initialize_process_core_affinity(proc_1, 1).unwrap();
    Process::initialize_process_core_affinity(proc_2, 2).unwrap();

    Process::start(proc_1).unwrap();
    Process::start(proc_2).unwrap();

    let code = Partition::set_partition_mode(OperatingMode::Normal);
    panic!("SET_PARTITION_MODE: {:?}", code);
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if let Some(location) = info.location() {
        error!(
            "panicked at {}:{} {}",
            location.file(),
            location.line(),
            info.message().unwrap(),
        );
    } else {
        error!("panicked: {}", info.message().unwrap());
    }

    sys_debug_halt();
}
