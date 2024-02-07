use alloc::{boxed::Box, format, sync::Arc};
use core::{future::Future, ops::Deref, pin::Pin};
use jrinx_apex::*;
use jrinx_paging::GenericPageTable;
use jrinx_trap::{arch::Context, GenericContext};

use jrinx_addr::VirtAddr;
use jrinx_config::PAGE_SIZE;
use jrinx_error::{InternalError, Result};
use jrinx_hal::{Hal, Vm};
use jrinx_multitask::{
    executor::{Executor, ExecutorPriority},
    Task, TaskPriority,
};
use jrinx_serial_id_macro::SerialId;
use spin::RwLock;

use crate::{
    partition::{Partition, PartitionId},
    A653Entry,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, SerialId)]
pub struct ProcessId(ApexProcessId);

pub struct Process {
    identifier: ProcessId,
    name: ApexProcessName,
    index: Option<ApexProcessIndex>,
    partition_id: PartitionId,
    stack_top: VirtAddr,
    base_priority: ApexPriority,
    deadline: ApexDeadline,
    entry: A653Entry,
    period: ApexSystemTime,
    stack_size: ApexStackSize,
    time_capacity: ApexSystemTime,
    curr_priority: RwLock<ApexPriority>,
    deadline_time: RwLock<ApexSystemTime>,
    process_state: RwLock<ApexProcessState>,
    core_affinity: RwLock<Option<usize>>,
}

pub struct ProcessConfig {
    pub name: ApexProcessName,
    pub priority: ApexPriority,
    pub deadline: ApexDeadline,
    pub entry: A653Entry,
    pub period: ApexSystemTime,
    pub stack_size: ApexStackSize,
    pub time_capacity: ApexSystemTime,
}

impl From<ProcessId> for ApexProcessId {
    fn from(id: ProcessId) -> Self {
        id.0
    }
}

impl From<ApexProcessId> for ProcessId {
    fn from(value: ApexProcessId) -> Self {
        Self(value)
    }
}

impl Process {
    pub const MAX_PRIORITY: ApexPriority = ExecutorPriority::MAX as _;

    pub fn new(partition_id: PartitionId, config: &ProcessConfig) -> Result<Arc<Self>> {
        let partition = Partition::find_by_id(partition_id).unwrap();
        let stack_top = partition.allocate_stack(config.stack_size as _)?;
        let index = partition.next_index();

        let process = Arc::new(Self {
            identifier: ProcessId::new(),
            name: config.name,
            index: if index == 0 { None } else { Some(index as _) },
            partition_id,
            stack_top,
            base_priority: config.priority,
            deadline: config.deadline,
            entry: config.entry,
            period: config.period,
            stack_size: config.stack_size,
            time_capacity: config.time_capacity,
            curr_priority: RwLock::new(config.priority),
            deadline_time: RwLock::new(APEX_TIME_INFINITY),
            process_state: RwLock::new(ApexProcessState::Dormant),
            core_affinity: RwLock::new(None),
        });

        partition.register_process(process.clone());

        Ok(process)
    }

    pub fn new_init(partition_id: PartitionId) -> Result<Arc<Self>> {
        let partition = Partition::find_by_id(partition_id).unwrap();

        Self::new(
            partition_id,
            &ProcessConfig {
                name: format!("{:?}.i", partition.name())
                    .as_str()
                    .try_into()
                    .map_err(|_| InternalError::InvalidApexName)?,
                priority: 0,
                deadline: ApexDeadline::Soft,
                entry: partition.entry(),
                period: APEX_TIME_INFINITY,
                stack_size: PAGE_SIZE as _,
                time_capacity: APEX_TIME_INFINITY,
            },
        )
    }

    pub fn current() -> Option<Arc<Self>> {
        Executor::with_current(|ex| ex.ext().deref().downcast_ref().cloned()).ok()?
    }

    pub fn find_by_id(partition_id: PartitionId, identifier: ProcessId) -> Option<Arc<Self>> {
        Partition::find_by_id(partition_id)
            .and_then(|partition| partition.find_process_by_id(identifier))
    }

    pub fn find_by_name(partition_id: PartitionId, name: &ApexProcessName) -> Option<Arc<Self>> {
        Partition::find_by_id(partition_id)
            .and_then(|partition| partition.find_process_by_name(name))
    }

    pub fn identifier(&self) -> ProcessId {
        self.identifier
    }

    pub fn name(&self) -> ApexProcessName {
        self.name
    }

    pub fn index(&self) -> Option<ApexProcessIndex> {
        self.index
    }

    pub fn partition_id(&self) -> PartitionId {
        self.partition_id
    }

    pub fn stack_top(&self) -> VirtAddr {
        self.stack_top
    }

    pub fn stack_size(&self) -> ApexStackSize {
        self.stack_size
    }

    pub fn period(&self) -> ApexSystemTime {
        self.period
    }

    pub fn time_capacity(&self) -> ApexSystemTime {
        self.time_capacity
    }

    pub fn entry(&self) -> A653Entry {
        self.entry
    }

    pub fn base_priority(&self) -> ApexPriority {
        self.base_priority
    }

    pub fn curr_priority(&self) -> ApexPriority {
        *self.curr_priority.read()
    }

    pub fn set_curr_priority(&self, priority: ApexPriority) {
        *self.curr_priority.write() = priority;
    }

    pub fn deadline_time(&self) -> ApexSystemTime {
        *self.deadline_time.read()
    }

    pub fn set_deadline_time(&self, time: ApexSystemTime) {
        *self.deadline_time.write() = time;
    }

    pub fn process_state(&self) -> ApexProcessState {
        *self.process_state.read()
    }

    pub fn set_process_state(&self, state: ApexProcessState) {
        *self.process_state.write() = state;
    }

    pub fn core_affinity(&self) -> Option<usize> {
        *self.core_affinity.read()
    }

    pub fn set_core_affinity(&self, cpu_id: Option<usize>) {
        *self.core_affinity.write() = cpu_id;
    }

    pub fn status(&self) -> ApexProcessStatus {
        ApexProcessStatus {
            attributes: ApexProcessAttribute {
                base_priority: self.base_priority,
                deadline: self.deadline,
                entry_point: match self.entry {
                    A653Entry::User(addr) =>
                    // Safety:
                    //   The function returned from `transmute(addr)` will not be called in kernel mode.
                    //   This transmutation is just for type compatibility.
                    unsafe { core::mem::transmute(addr) },
                    A653Entry::Kern(func) => func,
                },
                name: self.name,
                period: self.period,
                stack_size: self.stack_size,
                time_capacity: self.time_capacity,
            },
            current_priority: *self.curr_priority.read(),
            deadline_time: *self.deadline_time.read(),
            process_state: *self.process_state.read(),
        }
    }

    pub fn gen_executor<H, F>(
        self: &Arc<Self>,
        proc_runner: ProcessRunner<H, F>,
    ) -> Result<Pin<Box<Executor>>>
    where
        H: Fn(usize, [usize; 7]) -> F + Send + Sync + 'static,
        F: Future<Output = Result<usize>> + Send + 'static,
    {
        Ok(Executor::new_with_ext(
            ExecutorPriority::new(
                self.base_priority()
                    .try_into()
                    .map_err(|_| InternalError::InvalidApexPriority)?,
            ),
            Task::new(proc_runner.run(self.clone()), TaskPriority::default()),
            self.clone(),
        ))
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        if let Some(partition) = Partition::find_by_id(self.partition_id) {
            partition.deallocate_stack(self.stack_top).unwrap();
            hal!().vm().sync_all();
        }
    }
}

pub struct ProcessRunner<H, F>
where
    H: Fn(usize, [usize; 7]) -> F,
    F: Future<Output = Result<usize>>,
{
    pub syscall: H,
}

impl<H, F> ProcessRunner<H, F>
where
    H: Fn(usize, [usize; 7]) -> F,
    F: Future<Output = Result<usize>>,
{
    pub async fn run(self, process: Arc<Process>) {
        debug!("run process: {:?}", process.name());

        match process.entry() {
            A653Entry::Kern(_) => todo!(),
            A653Entry::User(entry) => self.user_run(process, entry).await,
        }
    }

    async fn user_run(self, process: Arc<Process>, entry: usize) {
        Partition::find_by_id(process.partition_id())
            .unwrap()
            .pt_sync();

        hal!().vm().enable(
            Partition::find_by_id(process.partition_id())
                .unwrap()
                .pt_read()
                .addr(),
        );
        hal!().vm().sync_all();

        let mut ctx = Context::default();
        ctx.user_setup(entry, process.stack_top().as_usize());

        loop {
            Partition::find_by_id(process.partition_id())
                .unwrap()
                .pt_sync();

            ctx.run();
            trace!("process trap: {:?}", ctx.trap_reason());

            Partition::find_by_id(process.partition_id())
                .unwrap()
                .pt_sync();

            self.user_handle_trap(&mut ctx).await;
        }
    }
    async fn user_handle_trap(&self, ctx: &mut Context) {
        let reason = ctx.trap_reason();
        match reason {
            jrinx_trap::TrapReason::SystemCall => {
                ctx.syscall_ret(
                    (self.syscall)(ctx.syscall_num(), ctx.syscall_args())
                        .await
                        .unwrap(), // TODO: handle error
                );
                ctx.pc_advance();
            }
            _ => unimplemented!("{:#x?}", ctx),
        }
    }
}
