use alloc::{collections::BTreeMap, sync::Arc};
use spin::{Lazy, Mutex};

use crate::{
    arch::task::{self, SwitchInfo},
    cpudata,
};

use super::{Task, TaskId};

pub trait Scheduler {
    fn insert(&mut self, task: Task);

    fn remove(&mut self, task_id: TaskId);

    fn sched(&mut self) -> Option<Arc<Task>>;
}

pub type GlobalScheduler = PreemptiveScheduler;

pub struct PreemptiveScheduler {
    items: BTreeMap<usize, Arc<Task>>,
}

impl PreemptiveScheduler {
    pub const fn new() -> Self {
        Self {
            items: BTreeMap::new(),
        }
    }
}

impl Scheduler for PreemptiveScheduler {
    fn insert(&mut self, task: Task) {
        self.items.insert(task.get_priority(), Arc::new(task));
    }

    fn remove(&mut self, task_id: TaskId) {
        self.items.retain(|_, task| task.ident != task_id)
    }

    fn sched(&mut self) -> Option<Arc<Task>> {
        self.items.last_entry().map(|o| o.get().clone())
    }
}

static GLOBAL_SCHEDULER: Lazy<Arc<Mutex<GlobalScheduler>>> =
    Lazy::new(|| Arc::new(Mutex::new(GlobalScheduler::new())));

pub fn with_global_scheduler(f: impl FnOnce(&mut GlobalScheduler)) {
    f(&mut *GLOBAL_SCHEDULER.clone().lock());
}

pub fn global_sched_start() -> ! {
    let next_task = global_get_next_task();
    let switch_info = switch_info_of_task(&next_task);
    assert!(cpudata::get_current_task().is_none());
    cpudata::set_current_task(next_task.clone());
    unsafe {
        task::switch::task_start(switch_info);
    }
}

pub fn global_destroy() -> ! {
    with_global_scheduler(|scheduler| {
        scheduler.remove(cpudata::clr_current_task().unwrap().get_ident())
    });
    let next_task = global_get_next_task();
    let next_switch_info = switch_info_of_task(&next_task);
    cpudata::set_current_task(next_task);
    unsafe {
        task::switch::task_continue(next_switch_info);
    }
}

pub fn global_switch() {
    let next_task = global_get_next_task();
    let next_switch_info = switch_info_of_task(&next_task);
    let prev_task = cpudata::clr_current_task().unwrap();
    let prev_switch_info = switch_info_of_task(&prev_task);
    cpudata::set_current_task(next_task);
    unsafe {
        task::switch::task_switch(prev_switch_info, next_switch_info);
    }
}

fn global_get_next_task() -> Arc<Task> {
    let mut scheduler = GLOBAL_SCHEDULER.lock();
    scheduler.sched().expect("no schedulable task")
}

fn switch_info_of_task(task: &Arc<Task>) -> *mut SwitchInfo {
    let mut switch_info = task.switch_info.write();
    &mut *switch_info as *mut _
}
