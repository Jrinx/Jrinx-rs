use core::pin::Pin;

use alloc::{boxed::Box, vec::Vec};
use spin::Mutex;

use crate::{
    arch,
    task::{executor::Executor, runtime::Runtime},
};

#[repr(align(4096))]
struct CpuData {
    runtime: Mutex<Pin<Box<Runtime>>>,
}

static mut CPU_DATA: Vec<CpuData> = Vec::new();

pub fn init() {
    let nproc = arch::cpus::nproc().unwrap();
    for _ in 0..nproc {
        unsafe {
            CPU_DATA.push(CpuData {
                runtime: Mutex::new(Runtime::new()),
            });
        }
    }
}

pub fn with_cpu_runtime<F, R>(f: F) -> R
where
    F: FnOnce(&mut Runtime) -> R,
{
    let mut runtime = unsafe { CPU_DATA[arch::cpu::id()].runtime.lock() };
    f(&mut runtime)
}

pub fn with_cpu_bootstrap_executor<F, R>(f: F) -> R
where
    F: FnOnce(&mut Pin<Box<Executor>>) -> R,
{
    with_cpu_runtime(|rt| rt.with_bootstrap_executor(f).unwrap())
}

pub fn with_cpu_executor<F, R>(f: F) -> R
where
    F: FnOnce(&mut Pin<Box<Executor>>) -> R,
{
    with_cpu_runtime(|rt| rt.with_current_executor(f).unwrap())
}
