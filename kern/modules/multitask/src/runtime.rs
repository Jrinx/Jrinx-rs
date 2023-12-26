use core::{future::Future, pin::Pin, sync::atomic::AtomicUsize, time::Duration};

use alloc::{
    boxed::Box,
    collections::{BTreeMap, VecDeque},
    vec::Vec,
};
use jrinx_addr::VirtAddr;
use jrinx_error::{InternalError, Result};
use jrinx_hal::{Cpu, Hal, HaltReason, Interrupt};
use jrinx_percpu::percpu;
use jrinx_timed_event::{TimedEvent, TimedEventHandler, TimedEventTracker};
use mtxgroup::MutexGroup;
use spin::{Mutex, Once};

use crate::{
    arch::{self, SwitchContext},
    executor::{Executor, ExecutorPriority},
    inspector::{Inspector, InspectorId, InspectorStatus},
    Task, TaskPriority,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RuntimeStatus {
    Init,
    Idle,
    Running(InspectorId),
    Endpoint,
}

pub struct Runtime {
    cpu_id: usize,
    inspector_registry: BTreeMap<InspectorId, Inspector>,
    inspector_queue: VecDeque<InspectorId>,
    status: RuntimeStatus,
    sched_table: Option<RuntimeSchedTable>,
    switch_context: SwitchContext,
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
    pub fn new(root_inspector: Inspector) -> Mutex<Pin<Box<Self>>> {
        let mut runtime = Box::pin(Self {
            cpu_id: hal!().cpu().id(),
            inspector_registry: BTreeMap::new(),
            inspector_queue: VecDeque::new(),
            status: RuntimeStatus::Init,
            sched_table: None,
            switch_context: SwitchContext::new_runtime(),
        });

        runtime.register_inspector(root_inspector).unwrap();

        Mutex::new(runtime)
    }

    pub fn enact_sched_table(&mut self, sched_table: RuntimeSchedTable) -> Result<()> {
        if self.sched_table.is_some() {
            return Err(InternalError::DuplicateRuntimeSchedTable);
        }
        self.sched_table = Some(sched_table);
        Ok(())
    }

    pub fn revoke_sched_table(&mut self) -> Result<RuntimeSchedTable> {
        self.sched_table
            .take()
            .ok_or(InternalError::InvalidRuntimeSchedTable)
    }

    pub fn register_inspector(&mut self, inspector: Inspector) -> Result<()> {
        let id = inspector.id();
        self.inspector_registry
            .try_insert(id, inspector)
            .map_err(|_| InternalError::DuplicateInspectorId)?;
        self.inspector_queue.push_back(id);
        Ok(())
    }

    pub fn unregister_inspector(&mut self, id: InspectorId) -> Result<()> {
        self.inspector_registry
            .remove(&id)
            .ok_or(InternalError::InvalidInspectorId)?;
        Ok(())
    }

    pub fn with_current_try_lock<F, R>(f: F) -> Result<R>
    where
        F: FnOnce(&mut Runtime) -> R,
    {
        hal!().interrupt().with_saved_off(|| {
            RUNTIME.with_ref(|rt| {
                if let Some(rt) = rt.get() {
                    match rt.try_lock() {
                        Some(ref mut rt) => Ok(f(rt)),
                        None => Err(InternalError::BusyLock),
                    }
                } else {
                    Err(InternalError::InvalidRuntimeStatus)
                }
            })
        })
    }

    pub fn with_current<F, R>(f: F) -> Result<R>
    where
        F: FnOnce(&mut Runtime) -> R,
    {
        hal!().interrupt().with_saved_off(|| {
            RUNTIME.with_ref(|rt| {
                if let Some(rt) = rt.get() {
                    Ok(f(&mut rt.lock()))
                } else {
                    Err(InternalError::InvalidRuntimeStatus)
                }
            })
        })
    }

    pub fn start() -> ! {
        debug!("runtime started running all inspectors");

        loop {
            if Runtime::with_current(|rt| rt.sched_table.is_some()).unwrap() {
                Runtime::run_with_sched_table();
                if Runtime::with_current(|rt| rt.sched_table.is_none()).unwrap() {
                    continue;
                }
            } else {
                Runtime::run_without_sched_table();
                if Runtime::with_current(|rt| rt.sched_table.is_some()).unwrap() {
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
        let runtime_switch_ctx = Runtime::with_current(|rt| rt.switch_context_addr()).unwrap();
        let executor_switch_ctx = Executor::with_current(|ex| ex.switch_context()).unwrap();
        unsafe {
            arch::switch(
                executor_switch_ctx.as_usize(),
                runtime_switch_ctx.as_usize(),
            );
        }
    }

    pub(crate) fn status(&self) -> RuntimeStatus {
        self.status
    }

    pub(crate) fn with_inspector<F, R>(&mut self, id: InspectorId, f: F) -> Result<R>
    where
        F: FnOnce(&mut Inspector) -> R,
    {
        let inspector = self
            .inspector_registry
            .get_mut(&id)
            .ok_or(InternalError::InvalidInspectorId)?;
        Ok(f(inspector))
    }

    fn set_current_inspector(&mut self, id: Option<InspectorId>) {
        if let Some(id) = id {
            match self.status {
                RuntimeStatus::Running(ref mut inspector_id) => {
                    *inspector_id = id;
                }
                _ => {
                    self.status = RuntimeStatus::Running(id);
                }
            }
        } else {
            self.status = RuntimeStatus::Idle;
        }
    }

    fn pop_inspector(&mut self) -> Option<InspectorId> {
        self.inspector_queue.pop_front()
    }

    fn push_inspector(&mut self, id: InspectorId) -> Result<()> {
        if !self.inspector_registry.contains_key(&id) {
            return Err(InternalError::InvalidInspectorId);
        }
        self.inspector_queue.push_back(id);
        Ok(())
    }

    fn switch_context_addr(&self) -> VirtAddr {
        VirtAddr::new(&self.switch_context as *const _ as usize)
    }

    fn sched_table_start(&self) -> Result<()> {
        self.sched_table
            .as_ref()
            .map(|table| table.start())
            .ok_or(InternalError::InvalidRuntimeSchedTable)
    }

    fn sched_table_next(&self) -> Option<RuntimeSchedTableEntry> {
        self.sched_table.as_ref().map(|table| table.sched_next())
    }

    fn run_with_sched_table() {
        let runtime_switch_ctx = Runtime::with_current(|rt| rt.switch_context_addr()).unwrap();

        Runtime::with_current(|rt| rt.sched_table_start().unwrap()).unwrap();

        while let Some(entry) = Runtime::with_current(|rt| rt.sched_table_next()).unwrap() {
            trace!("switch into inspector {:?}", entry.inspector_id);

            Runtime::with_current(|rt| {
                rt.set_current_inspector(Some(entry.inspector_id));
            })
            .unwrap();

            hal!().interrupt().with_saved_on(|| {
                Inspector::run(runtime_switch_ctx);
            });

            Runtime::with_current(|rt| {
                rt.set_current_inspector(None);
            })
            .unwrap();

            trace!("switch from inspector {:?}", entry.inspector_id);
        }
    }

    fn run_without_sched_table() {
        let runtime_switch_ctx = Runtime::with_current(|rt| rt.switch_context_addr()).unwrap();

        while let Some(inspector_id) = Runtime::with_current(|rt| {
            rt.sched_table
                .is_none()
                .then(|| rt.pop_inspector())
                .flatten()
        })
        .unwrap()
        {
            trace!("switch into inspector {:?}", inspector_id);

            Runtime::with_current(|rt| rt.set_current_inspector(Some(inspector_id))).unwrap();

            hal!().interrupt().with_saved_on(|| {
                Inspector::run(runtime_switch_ctx);
            });

            Runtime::with_current(|rt| {
                rt.set_current_inspector(None);
            })
            .unwrap();

            trace!("switch from inspector {:?}", inspector_id);

            if Runtime::with_current(|rt| {
                rt.with_inspector(inspector_id, |is| {
                    is.is_empty() && is.status() == InspectorStatus::Idle
                })
                .unwrap()
            })
            .unwrap()
            {
                Runtime::with_current(|rt| rt.unregister_inspector(inspector_id).unwrap()).unwrap();
            } else {
                Runtime::with_current(|rt| rt.push_inspector(inspector_id).unwrap()).unwrap();
            }
        }
    }

    fn halt_if_all_finished_or_ipi() {
        let runtimes = MutexGroup::new(RUNTIME.iter().filter_map(|rt| rt.get()));
        let guards = runtimes.lock();

        if guards.iter().count() == 1
            || guards
                .iter()
                .all(|guard| guard.status == RuntimeStatus::Endpoint)
        {
            hal!().halt(HaltReason::NormalExit);
        } else {
            if let Some(cpu_id) = guards
                .iter()
                .filter_map(|guard| {
                    (guard.status == RuntimeStatus::Endpoint).then_some(guard.cpu_id)
                })
                .min()
            {
                if cpu_id != hal!().cpu().id() {
                    hal!().interrupt().send_ipi(&[cpu_id]);
                }
            }

            guards
                .into_iter()
                .find(|guard| guard.cpu_id == hal!().cpu().id())
                .unwrap()
                .status = RuntimeStatus::Endpoint;
        }
    }
}

unsafe impl Send for Runtime {}

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
static RUNTIME: Once<Mutex<Pin<Box<Runtime>>>> = Once::new();

pub fn init(future: impl Future<Output = ()> + 'static) {
    RUNTIME
        .as_ref()
        .try_call_once::<_, ()>(|| {
            Ok(Runtime::new(Inspector::new(Executor::new(
                ExecutorPriority::default(),
                Task::new(future, TaskPriority::default()),
            ))))
        })
        .unwrap();
}
