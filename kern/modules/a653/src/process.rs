use a653rs::bindings::{ProcessIndex, ProcessName, StackSize};
use alloc::sync::Arc;
use spin::Mutex;

use jrinx_addr::VirtAddr;
use jrinx_error::Result;
use jrinx_hal::{Hal, Vm};
use jrinx_serial_id_macro::SerialId;
use jrinx_trap::{arch::Context, GenericContext};

use crate::partition::{Partition, PartitionId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, SerialId)]
pub struct ProcessId(a653rs::bindings::ProcessId);

pub struct Process {
    identifier: ProcessId,
    name: ProcessName,
    index: Option<ProcessIndex>,
    partition_id: PartitionId,
    stack_top: VirtAddr,
    stack_size: StackSize,
    entry: usize,
    context: Mutex<Context>,
}

pub struct ProcessConfig {
    name: ProcessName,
    partition_id: PartitionId,
    stack_size: StackSize,
    entry: usize,
}

impl Process {
    pub fn new(config: &ProcessConfig) -> Result<Arc<Self>> {
        let partition = Partition::find(config.partition_id).unwrap();

        let stack_top = partition.allocate_stack(config.stack_size as _)?;

        hal!().vm().sync_all();

        let index = partition.next_index();

        let process = Arc::new(Self {
            identifier: ProcessId::new(),
            name: config.name,
            index: if index == 0 { None } else { Some(index as _) },
            partition_id: config.partition_id,
            stack_top,
            stack_size: config.stack_size,
            entry: config.entry,
            context: Mutex::new(Context::default()),
        });

        Ok(process)
    }

    pub fn identifier(&self) -> ProcessId {
        self.identifier
    }

    pub fn name(&self) -> ProcessName {
        self.name
    }

    pub fn index(&self) -> Option<ProcessIndex> {
        self.index
    }

    pub fn partition_id(&self) -> PartitionId {
        self.partition_id
    }

    pub fn stack_top(&self) -> VirtAddr {
        self.stack_top
    }

    pub fn stack_size(&self) -> StackSize {
        self.stack_size
    }

    pub fn entry(&self) -> usize {
        self.entry
    }

    pub fn run(&self) {
        // TODO: sync page table
        self.context.lock().run();
        // TODO: sync page table
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        Partition::find(self.partition_id)
            .unwrap()
            .deallocate_stack(self.stack_top)
            .unwrap();

        hal!().vm().sync_all();
    }
}

impl ProcessConfig {
    pub fn new(
        name: ProcessName,
        partition_id: PartitionId,
        stack_size: StackSize,
        entry: usize,
    ) -> Self {
        Self {
            name,
            partition_id,
            stack_size,
            entry,
        }
    }
}
