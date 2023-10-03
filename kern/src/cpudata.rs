use core::pin::Pin;

use alloc::{boxed::Box, vec::Vec};
use spin::Mutex;

use crate::{arch, task::executor::Executor};

#[repr(align(4096))]
struct CpuData {
    executor: Mutex<Pin<Box<Executor>>>,
}

static mut CPU_DATA: Vec<CpuData> = Vec::new();

pub fn init() {
    let nproc = arch::cpus::nproc().unwrap();
    for _ in 0..nproc {
        unsafe {
            CPU_DATA.push(CpuData {
                executor: Mutex::new(Executor::new()),
            });
        }
    }
}

pub fn with_cpu_executor<R>(func: impl FnOnce(&mut Pin<Box<Executor>>) -> R) -> R {
    let mut executor = unsafe { CPU_DATA[arch::cpu::id()].executor.lock() };
    func(&mut executor)
}
