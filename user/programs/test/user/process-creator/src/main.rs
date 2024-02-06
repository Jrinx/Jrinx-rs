#![no_std]
#![no_main]
#![feature(panic_info_message)]

#[macro_use]
extern crate log;

use core::panic::PanicInfo;

use jrinx_abi::sysfn;
use jrlib_a653::prelude::*;

extern "C" fn entry() -> ! {
    info!("Hello World!");
    #[allow(clippy::empty_loop)]
    loop {}
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    jrlib_logging::init();

    let proc_1 = Process
        .create_process(&ApexProcessAttribute {
            period: APEX_TIME_INFINITY,
            time_capacity: APEX_TIME_INFINITY,
            entry_point: ApexSystemAddress::of(entry),
            stack_size: 4 * 4096,
            base_priority: ApexPriority::default(),
            deadline: ApexDeadline::Soft,
            name: "proc_1".try_into().unwrap(),
        })
        .unwrap();

    let proc_2 = Process
        .create_process(&ApexProcessAttribute {
            period: APEX_TIME_INFINITY,
            time_capacity: APEX_TIME_INFINITY,
            entry_point: ApexSystemAddress::of(entry),
            stack_size: 4 * 4096,
            base_priority: ApexPriority::default(),
            deadline: ApexDeadline::Soft,
            name: "proc_2".try_into().unwrap(),
        })
        .unwrap();

    Process.initialize_process_core_affinity(proc_1, 1).unwrap();
    Process.initialize_process_core_affinity(proc_2, 2).unwrap();

    Process.start(proc_1).unwrap();
    Process.start(proc_2).unwrap();

    let code = Partition.set_partition_mode(ApexOperatingMode::Normal);
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

    sysfn::sys_debug_halt();
}
