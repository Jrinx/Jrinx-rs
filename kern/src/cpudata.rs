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
fn cpu_data() -> &'static mut CpuData {
    unsafe { &mut CPU_DATA[arch::cpu::id()] }
}

pub fn get_current_task() -> Option<Arc<Task>> {
    cpu_data().task.clone()
}

pub fn set_current_task(task: Arc<Task>) {
    cpu_data().task.replace(task);
}

pub fn clr_current_task() -> Option<Arc<Task>> {
    cpu_data().task.take()
}
