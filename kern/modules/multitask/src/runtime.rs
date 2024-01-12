use core::{cell::SyncUnsafeCell, future::Future, sync::atomic::AtomicUsize, time::Duration};

use alloc::{
    collections::{BTreeMap, VecDeque},
    vec::Vec,
};
use jrinx_addr::VirtAddr;
use jrinx_error::{InternalError, Result};
use jrinx_hal::{Cpu, Hal, HaltReason, Interrupt};
use jrinx_percpu::percpu;
use jrinx_timed_event::{TimedEvent, TimedEventHandler, TimedEventTracker};
use mtxgroup::MutexGroup;
use spin::{Mutex, RwLock};

use crate::{
    arch::{self, SwitchContext},
    executor::{Executor, ExecutorPriority},
    inspector::{Inspector, InspectorId, InspectorStatus},
    Task, TaskPriority,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeStatus {
    Unused,
    Init,
    Idle,
    Running(InspectorId),
    Endpoint,
}

pub struct Runtime {
    scheduler: RwLock<RuntimeInspectorScheduler>,
    status: Mutex<RuntimeStatus>,
    switch_context: SyncUnsafeCell<SwitchContext>,
}

struct RuntimeInspectorScheduler {
    registry: BTreeMap<InspectorId, Inspector>,
    queue: VecDeque<InspectorId>,
    sched_table: Option<RuntimeSchedTable>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuntimeSchedTableEntry {
    pub inspector_id: InspectorId,
    pub offset: Duration,
    pub period: Duration,
    pub duration: Duration,
}

pub struct RuntimeSchedTable {
    frame_size: Duration,
    table: Vec<RuntimeSchedTableEntry>,
    next: AtomicUsize,
    datum: Mutex<Duration>,
    events: Mutex<VecDeque<TimedEventTracker>>,
}

impl Runtime {
    pub const fn new() -> Self {
        Self {
            scheduler: RwLock::new(RuntimeInspectorScheduler {
                registry: BTreeMap::new(),
                queue: VecDeque::new(),
                sched_table: None,
            }),
            status: Mutex::new(RuntimeStatus::Unused),
            switch_context: SyncUnsafeCell::new(SwitchContext::new_runtime()),
        }
    }

    pub fn init(&self) {
        *self.status.lock() = RuntimeStatus::Init;
    }

    pub fn enact_sched_table(&self, sched_table: RuntimeSchedTable) -> Result<()> {
        let mut scheduler = self.scheduler.write();
        if scheduler.sched_table.is_some() {
            return Err(InternalError::DuplicateRuntimeSchedTable);
        }
        scheduler.queue.retain(|&id| {
            sched_table
                .table
                .iter()
                .all(|entry| entry.inspector_id != id)
        });
        scheduler.sched_table = Some(sched_table);
        Ok(())
    }

    pub fn revoke_sched_table(&self) -> Result<RuntimeSchedTable> {
        self.scheduler
            .write()
            .sched_table
            .take()
            .ok_or(InternalError::InvalidRuntimeSchedTable)
    }

    pub fn register(&self, inspector: Inspector) -> Result<()> {
        let id = inspector.id();
        let mut inspectors = self.scheduler.write();
        inspectors
            .registry
            .try_insert(id, inspector)
            .map_err(|_| InternalError::DuplicateInspectorId)?;
        inspectors.queue.push_back(id);
        Ok(())
    }

    pub fn unregister(&self, id: InspectorId) -> Result<()> {
        self.scheduler
            .write()
            .registry
            .remove(&id)
            .ok_or(InternalError::InvalidInspectorId)?;
        Ok(())
    }

    pub fn with_current<F, R>(f: F) -> R
    where
        F: FnOnce(&Runtime) -> R,
    {
        hal!()
            .interrupt()
            .with_saved_off(|| RUNTIME.with_ref(|rt| f(rt)))
    }

    pub fn start() -> ! {
        debug!("runtime started running all inspectors");

        loop {
            if Runtime::with_current(|rt| rt.scheduler.read().sched_table.is_some()) {
                Runtime::run_with_sched_table();
                if Runtime::with_current(|rt| rt.scheduler.read().sched_table.is_none()) {
                    continue;
                }
            } else {
                Runtime::run_without_sched_table();
                if Runtime::with_current(|rt| rt.scheduler.read().sched_table.is_some()) {
                    continue;
                }
            }

            debug!("runtime finished running all inspectors");

            Runtime::halt_if_all_finished_or_ipi();

            debug!("runtime send ipi and wait");
            hal!().interrupt().with_saved_on(|| {
                hal!().interrupt().wait();
            });
        }
    }

    pub fn switch_yield() {
        let runtime_switch_ctx = Runtime::with_current(|rt| rt.switch_context_addr());
        let executor_switch_ctx = Executor::with_current(|ex| ex.switch_context()).unwrap();
        unsafe {
            arch::switch(
                executor_switch_ctx.as_usize(),
                runtime_switch_ctx.as_usize(),
            );
        }
    }

    pub fn status(&self) -> RuntimeStatus {
        *self.status.lock()
    }

    pub(crate) fn with_inspector<F, R>(&self, id: InspectorId, f: F) -> Result<R>
    where
        F: FnOnce(&Inspector) -> R,
    {
        Ok(f(self
            .scheduler
            .read()
            .registry
            .get(&id)
            .ok_or(InternalError::InvalidInspectorId)?))
    }

    fn set_current_inspector(&self, id: Option<InspectorId>) {
        let mut status = self.status.lock();

        if let Some(id) = id {
            match *status {
                RuntimeStatus::Running(ref mut inspector_id) => {
                    *inspector_id = id;
                }
                _ => {
                    *status = RuntimeStatus::Running(id);
                }
            }
        } else {
            *status = RuntimeStatus::Idle;
        }
    }

    fn pop_front(&self) -> Option<InspectorId> {
        self.scheduler.write().queue.pop_front()
    }

    fn push_back(&self, id: InspectorId) -> Result<()> {
        let mut scheduler = self.scheduler.write();

        if !scheduler.registry.contains_key(&id) {
            return Err(InternalError::InvalidInspectorId);
        }
        scheduler.queue.push_back(id);
        Ok(())
    }

    fn switch_context_addr(&self) -> VirtAddr {
        VirtAddr::new(self.switch_context.get() as *const _ as usize)
    }

    fn sched_table_start(&self) -> Result<()> {
        self.scheduler
            .read()
            .sched_table
            .as_ref()
            .map(|table| table.start())
            .ok_or(InternalError::InvalidRuntimeSchedTable)
    }

    fn sched_table_next(&self) -> Option<RuntimeSchedTableEntry> {
        self.scheduler
            .read()
            .sched_table
            .as_ref()
            .map(|table| table.sched_next())
    }

    fn run_with_sched_table() {
        let runtime_switch_ctx = Runtime::with_current(|rt| rt.switch_context_addr());

        Runtime::with_current(|rt| rt.sched_table_start().unwrap());

        while let Some(entry) = Runtime::with_current(|rt| rt.sched_table_next()) {
            trace!("switch into inspector {:?}", entry.inspector_id);

            Runtime::with_current(|rt| {
                rt.set_current_inspector(Some(entry.inspector_id));
            });

            hal!().interrupt().with_saved_on(|| {
                Inspector::run(runtime_switch_ctx);
            });

            Runtime::with_current(|rt| {
                rt.set_current_inspector(None);
            });

            trace!("switch from inspector {:?}", entry.inspector_id);
        }
    }

    fn run_without_sched_table() {
        let runtime_switch_ctx = Runtime::with_current(|rt| rt.switch_context_addr());

        while let Some(inspector_id) = Runtime::with_current(|rt| {
            if rt.scheduler.read().sched_table.is_none() {
                rt.pop_front()
            } else {
                None
            }
        }) {
            trace!("switch into inspector {:?}", inspector_id);

            Runtime::with_current(|rt| rt.set_current_inspector(Some(inspector_id)));

            hal!().interrupt().with_saved_on(|| {
                Inspector::run(runtime_switch_ctx);
            });

            Runtime::with_current(|rt| {
                rt.set_current_inspector(None);
            });

            trace!("switch from inspector {:?}", inspector_id);

            if Runtime::with_current(|rt| {
                rt.with_inspector(inspector_id, |is| {
                    is.is_empty() && is.status() == InspectorStatus::Idle
                })
                .unwrap()
            }) {
                Runtime::with_current(|rt| rt.unregister(inspector_id).unwrap());
            } else {
                Runtime::with_current(|rt| rt.push_back(inspector_id).unwrap());
            }
        }
    }

    fn halt_if_all_finished_or_ipi() {
        let status = MutexGroup::new(RUNTIME.iter().map(|rt| &rt.status));
        let guards = status.lock();

        if guards.iter().count() == 1
            || guards
                .iter()
                .filter(|&guard| **guard != RuntimeStatus::Unused)
                .all(|guard| **guard == RuntimeStatus::Endpoint)
        {
            hal!().halt(HaltReason::NormalExit);
        } else {
            if let Some(cpu_id) = guards
                .iter()
                .zip(0..)
                .filter_map(|(guard, cpu_id)| {
                    (**guard == RuntimeStatus::Endpoint).then_some(cpu_id)
                })
                .min()
            {
                if cpu_id != hal!().cpu().id() {
                    hal!().interrupt().send_ipi(&[cpu_id]);
                }
            }

            *guards
                .into_iter()
                .zip(0..)
                .find_map(|(guard, cpu_id)| (cpu_id == hal!().cpu().id()).then_some(guard))
                .unwrap() = RuntimeStatus::Endpoint;
        }
    }
}

impl RuntimeSchedTable {
    pub fn new(
        frame_size: Duration,
        table: impl Iterator<Item = RuntimeSchedTableEntry>,
    ) -> Result<Self> {
        let mut table: Vec<RuntimeSchedTableEntry> = table.collect::<Vec<_>>();
        table.sort_unstable_by_key(|entry| entry.offset);
        let sched_table = Self {
            frame_size,
            table,
            next: AtomicUsize::new(0),
            datum: Mutex::default(),
            events: Mutex::default(),
        };
        if !sched_table.valid() {
            return Err(InternalError::InvalidRuntimeSchedTable);
        }
        Ok(sched_table)
    }

    pub fn start(&self) {
        *self.datum.lock() = hal!().cpu().get_time();
    }

    pub(crate) fn sched_next(&self) -> RuntimeSchedTableEntry {
        self.events.lock().retain(|event| !event.retired());

        let next = self.table[self
            .next
            .fetch_add(1, core::sync::atomic::Ordering::Relaxed)];

        if hal!().cpu().get_time() < self.get_datum() + next.offset {
            self.events.lock().push_back(TimedEvent::create(
                self.get_datum() + next.offset,
                TimedEventHandler::new(|| {}, || {}),
            ));
            hal!().interrupt().wait();
        }

        self.events.lock().push_back(TimedEvent::create(
            self.get_datum() + next.offset + next.duration,
            TimedEventHandler::new(
                || {
                    Inspector::with_current(|is| is.mark_pending().unwrap()).unwrap();
                    hal!().interrupt().with_saved_on(|| {
                        Runtime::switch_yield();
                    });
                },
                || {},
            ),
        ));

        if self.next.load(core::sync::atomic::Ordering::Relaxed) >= self.table.len() {
            self.next.store(0, core::sync::atomic::Ordering::Relaxed);
            self.set_datum(self.get_datum() + self.frame_size);
        }

        next
    }

    fn get_datum(&self) -> Duration {
        *self.datum.lock()
    }

    fn set_datum(&self, datum: Duration) {
        *self.datum.lock() = datum;
    }

    fn valid(&self) -> bool {
        for i in 0..self.table.len() {
            let entry = &self.table[i];
            if entry.offset >= self.frame_size {
                return false;
            }
            if i + 1 < self.table.len() {
                let next_entry = &self.table[i + 1];
                if entry.offset + entry.duration > next_entry.offset {
                    return false;
                }
            }
        }

        let mut table = self.table.clone();
        table.sort_unstable_by_key(|entry| (entry.inspector_id, entry.offset));

        for entries in table.group_by(|e1, e2| e1.inspector_id == e2.inspector_id) {
            if !match entries {
                [head, tail @ ..] => tail
                    .iter()
                    .all(|e| e.period == head.period && e.duration == head.duration),
                [] => false,
            } {
                return false;
            }
        }

        for entries in table.group_by(|e1, e2| e1.inspector_id == e2.inspector_id) {
            if !entries
                .iter()
                .chain(entries.iter().take(1))
                .map_windows(|&[e1, e2]| {
                    let actual_period = if e2.offset <= e1.offset {
                        e2.offset + self.frame_size - e1.offset
                    } else {
                        e2.offset - e1.offset
                    };
                    actual_period == e1.period
                })
                .all(|eq| eq)
            {
                return false;
            }
        }

        true
    }
}

impl Drop for RuntimeSchedTable {
    fn drop(&mut self) {
        hal!().interrupt().with_saved_off(|| {
            warn!("drop runtime sched table");
            for event in self.events.lock().iter() {
                if let Err(err) = event.cancel() {
                    warn!("failed to cancel timed event: {:?}", err);
                }
            }
        });
    }
}

#[percpu]
static RUNTIME: Runtime = Runtime::new();

pub fn init(future: impl Future<Output = ()> + Send + Sync + 'static) {
    RUNTIME
        .as_ref()
        .register(Inspector::new(Executor::new(
            ExecutorPriority::default(),
            Task::new(future, TaskPriority::default()),
        )))
        .unwrap();
}
