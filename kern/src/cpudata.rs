use alloc::{sync::Arc, vec::Vec};

use crate::{arch, task::Task};

#[repr(align(4096))]
struct CpuData {
    task: Option<Arc<Task>>,
}

static mut CPU_DATA: Vec<CpuData> = Vec::new();

pub fn init() {
    let nproc = arch::cpus::nproc().unwrap();
    info!("cpus nproc: {}", nproc);
    for _ in 0..nproc {
        unsafe {
            CPU_DATA.push(CpuData { task: None });
        }
    }
}

#[inline]
fn cpu_data() -> Option<&'static mut CpuData> {
    if arch::cpu::id() >= unsafe { CPU_DATA.len() } {
        None
    } else {
        Some(unsafe { &mut CPU_DATA[arch::cpu::id()] })
    }
}

#[inline]
pub fn get_current_task() -> Option<Arc<Task>> {
    cpu_data()?.task.clone()
}

#[inline]
pub fn set_current_task(task: Arc<Task>) {
    cpu_data().unwrap().task.replace(task);
}

#[inline]
pub fn clr_current_task() -> Option<Arc<Task>> {
    cpu_data()?.task.take()
}
