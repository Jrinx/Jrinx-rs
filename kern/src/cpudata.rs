use core::pin::Pin;

use alloc::{boxed::Box, vec::Vec};
use spin::Mutex;

use crate::{
    arch,
    error::{InternalError, Result},
    task::{
        executor::{Executor, ExecutorPriority},
        runtime::{
            inspector::{Inspector, InspectorMode},
            Runtime,
        },
        Task, TaskPriority,
    },
    time::TimedEventQueue,
    trap::interrupt,
};

#[repr(align(4096))]
struct CpuData {
    runtime: Mutex<Pin<Box<Runtime>>>,
    timed_event_queue: Mutex<TimedEventQueue>,
}

static mut CPU_DATA: Vec<CpuData> = Vec::new();

pub fn init() {
    let nproc = arch::cpus::nproc().unwrap();
    for _ in 0..nproc {
        unsafe {
            CPU_DATA.push(CpuData {
                runtime: Mutex::new(Runtime::new(Inspector::new(
                    InspectorMode::Bootstrap,
                    Executor::new(
                        ExecutorPriority::default(),
                        Task::new(super::master_init(), TaskPriority::default()),
                    ),
                ))),
                timed_event_queue: Mutex::new(TimedEventQueue::new()),
            });
        }
    }
}

fn cpu_data() -> Option<&'static CpuData> {
    if arch::cpu::id() >= unsafe { CPU_DATA.len() } {
        None
    } else {
        Some(unsafe { &CPU_DATA[arch::cpu::id()] })
    }
}

pub fn with_cpu_runtime<F, R>(f: F) -> Result<R>
where
    F: FnOnce(&mut Runtime) -> R,
{
    interrupt::with_interrupt_saved_off(|| {
        let mut runtime = cpu_data()
            .ok_or(InternalError::InvalidCpuId)?
            .runtime
            .lock();
        Ok(f(&mut runtime))
    })
}

pub fn with_cpu_inspector<F, R>(f: F) -> Result<R>
where
    F: FnOnce(&mut Inspector) -> R,
{
    with_cpu_runtime(|rt| rt.with_current_inspector(f))?
}

pub fn with_cpu_executor<F, R>(f: F) -> Result<R>
where
    F: FnOnce(&mut Pin<Box<Executor>>) -> R,
{
    with_cpu_inspector(|inspector| inspector.with_current_executor(f))?
}

pub fn with_cpu_timed_event_queue<F, R>(f: F) -> Result<R>
where
    F: FnOnce(&mut TimedEventQueue) -> R,
{
    interrupt::with_interrupt_saved_off(|| {
        Ok(f(&mut cpu_data()
            .ok_or(InternalError::InvalidCpuId)?
            .timed_event_queue
            .lock()))
    })
}
