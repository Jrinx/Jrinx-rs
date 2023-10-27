use alloc::{boxed::Box, vec::Vec};
use core::pin::Pin;
use jrinx_error::{InternalError, Result};
use spin::Mutex;

use crate::{
    arch,
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
    util::once_lock::OnceLock,
};

#[repr(align(4096))]
struct CpuData {
    runtime: Mutex<Pin<Box<Runtime>>>,
    timed_event_queue: Mutex<TimedEventQueue>,
}

unsafe impl Send for CpuData {}
unsafe impl Sync for CpuData {}

static CPU_DATA: OnceLock<Vec<CpuData>> = OnceLock::new();

pub fn init() {
    CPU_DATA
        .init(
            (0..arch::cpus::nproc().unwrap())
                .map(|_| CpuData {
                    runtime: Mutex::new(Runtime::new(Inspector::new(
                        InspectorMode::Bootstrap,
                        Executor::new(
                            ExecutorPriority::default(),
                            Task::new(super::master_init(), TaskPriority::default()),
                        ),
                    ))),
                    timed_event_queue: Mutex::new(TimedEventQueue::new()),
                })
                .collect::<Vec<_>>(),
        )
        .unwrap();
}

pub struct CpuDataVisitor {
    cpu_id: usize,
}

impl CpuDataVisitor {
    pub fn new() -> Self {
        CpuDataVisitor {
            cpu_id: arch::cpu::id(),
        }
    }

    pub fn id(self, cpu_id: usize) -> Self {
        CpuDataVisitor { cpu_id }
    }

    pub fn runtime<F, R>(self, f: F) -> Result<R>
    where
        F: FnOnce(&mut Runtime) -> R,
    {
        interrupt::with_interrupt_saved_off(|| {
            let mut runtime = self
                .cpu_data()
                .ok_or(InternalError::InvalidCpuId)?
                .runtime
                .lock();
            Ok(f(&mut runtime))
        })
    }

    pub fn inspector<F, R>(self, f: F) -> Result<R>
    where
        F: FnOnce(&mut Inspector) -> R,
    {
        self.runtime(|rt| rt.with_current_inspector(f))?
    }

    pub fn executor<F, R>(self, f: F) -> Result<R>
    where
        F: FnOnce(&mut Pin<Box<Executor>>) -> R,
    {
        self.inspector(|inspector| inspector.with_current_executor(f))?
    }

    pub fn timed_event_queue<F, R>(self, f: F) -> Result<R>
    where
        F: FnOnce(&mut TimedEventQueue) -> R,
    {
        interrupt::with_interrupt_saved_off(|| {
            Ok(f(&mut self
                .cpu_data()
                .ok_or(InternalError::InvalidCpuId)?
                .timed_event_queue
                .lock()))
        })
    }

    fn cpu_data(&self) -> Option<&'static CpuData> {
        let cpu_id = self.cpu_id;
        if cpu_id >= CPU_DATA.get().map(|v| v.len()).unwrap_or(0) {
            None
        } else {
            Some(&CPU_DATA.get().unwrap()[cpu_id])
        }
    }
}
