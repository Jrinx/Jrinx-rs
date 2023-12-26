use core::{alloc::Allocator, sync::atomic::AtomicUsize};

use a653rs::bindings::{
    ApexName, ApexPartitionStatus, ApexSystemTime, LockLevel, NumCores, OperatingMode, ProcessName,
    StartCondition, MIN_LOCK_LEVEL,
};
use alloc::{alloc::Global, collections::BTreeMap, format, string::String, sync::Arc};
use elf::{
    abi::{PF_R, PF_W, PF_X},
    endian::AnyEndian,
    ElfBytes,
};
use spin::{Mutex, RwLock};

use jrinx_addr::VirtAddr;
use jrinx_config::PAGE_SIZE;
use jrinx_error::{InternalError, Result};
use jrinx_hal::{hal, Cache, Hal, Vm};
use jrinx_loader::ElfLoader;
use jrinx_paging::{common::PageTable, GenericPagePerm, GenericPageTable, PagePerm};
use jrinx_phys_frame::PhysFrame;
use jrinx_serial_id_macro::SerialId;
use jrinx_stack_alloc::StackAllocator;
use jrinx_vmm::KERN_PAGE_TABLE;

use crate::process::{Process, ProcessConfig, ProcessId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, SerialId)]
pub struct PartitionId(a653rs::bindings::PartitionId);

pub struct Partition {
    identifier: PartitionId,
    name: ApexName,
    memory: PartitionMemory,
    page_table: RwLock<PageTable>,
    process_registry: RwLock<PartitionProcessRegistry>,
    stack_allocator: StackAllocator,
    next_index: AtomicUsize,
    period: ApexSystemTime,
    duration: ApexSystemTime,
    lock_level: LockLevel,
    operating_mode: OperatingMode,
    start_condition: StartCondition,
    num_assigned_cores: NumCores,
    entry: usize,
}

struct PartitionMemory {
    size: usize,
    free: Mutex<usize>,
}

struct PartitionProcessRegistry {
    registry: BTreeMap<ProcessId, Arc<Process>>,
    names: BTreeMap<ProcessName, ProcessId>,
}

#[derive(Debug, Clone, Copy)]
pub struct PartitionMemoryAllocator {
    partition_id: PartitionId,
}

pub struct PartitionConfig {
    name: ApexName,
    program: String,
    memory: usize,
    period: ApexSystemTime,
    duration: ApexSystemTime,
    num_cores: NumCores,
}

static PARTITIONS: RwLock<BTreeMap<PartitionId, Arc<Partition>>> = RwLock::new(BTreeMap::new());

impl Partition {
    pub fn new(config: &PartitionConfig) -> Result<Arc<Self>> {
        let program = jrinx_uprog::find(&config.program)?;
        let page_table = PageTable::new_from(&KERN_PAGE_TABLE.read())?;
        let partition_id = PartitionId::new();

        let stack_allocator = StackAllocator::new(
            (
                VirtAddr::new(jrinx_config::UPROG_STACK_REGION.addr),
                jrinx_config::UPROG_STACK_REGION.len,
            ),
            jrinx_config::PAGE_SIZE,
            move |addr| {
                let partition = Partition::find(partition_id).unwrap();
                partition.page_table.write().map(
                    addr,
                    PhysFrame::alloc_in(partition.allocator())?,
                    PagePerm::U | PagePerm::R | PagePerm::W,
                )?;
                Ok(())
            },
            move |addr| {
                let partition = Partition::find(partition_id).unwrap();
                partition.page_table.write().unmap(addr)?;
                Ok(())
            },
        );

        let partition = Arc::new(Self {
            identifier: partition_id,
            name: config.name,
            memory: PartitionMemory::new(config.memory),
            page_table: RwLock::new(page_table),
            process_registry: RwLock::new(PartitionProcessRegistry::new()),
            stack_allocator,
            next_index: AtomicUsize::new(0),
            period: config.period,
            duration: config.duration,
            lock_level: MIN_LOCK_LEVEL,
            operating_mode: OperatingMode::Idle,
            start_condition: StartCondition::NormalStart,
            num_assigned_cores: config.num_cores,
            entry: program.ehdr.e_entry as usize,
        });

        partition.load_program(&program)?;

        PARTITIONS
            .write()
            .insert(partition.identifier, partition.clone());

        let init_process = Process::new(&ProcessConfig::new(
            format!("{}.i", String::from_utf8_lossy(&partition.name))
                .as_bytes()
                .try_into()
                .map_err(|_| InternalError::InvalidName)?,
            partition.identifier,
            PAGE_SIZE as _,
            partition.entry,
        ))?;

        partition.process_registry.write().insert(init_process);

        Ok(partition)
    }

    pub fn find(id: PartitionId) -> Option<Arc<Self>> {
        PARTITIONS.read().get(&id).cloned()
    }

    pub fn name(&self) -> ApexName {
        self.name
    }

    pub fn memory_size(&self) -> usize {
        self.memory.size
    }

    pub fn memory_free(&self) -> usize {
        *self.memory.free.lock()
    }

    pub fn entry(&self) -> usize {
        self.entry
    }

    pub fn allocator(&self) -> PartitionMemoryAllocator {
        PartitionMemoryAllocator {
            partition_id: self.identifier,
        }
    }

    pub fn status(&self) -> ApexPartitionStatus {
        ApexPartitionStatus {
            period: self.period,
            duration: self.duration,
            identifier: self.identifier.0,
            lock_level: self.lock_level,
            operating_mode: self.operating_mode,
            start_condition: self.start_condition,
            num_assigned_cores: self.num_assigned_cores,
        }
    }

    pub fn allocate_stack(&self, stack_size: usize) -> Result<VirtAddr> {
        self.stack_allocator.allocate(stack_size)
    }

    pub fn deallocate_stack(&self, stack_top: VirtAddr) -> Result<()> {
        self.stack_allocator.deallocate(stack_top)
    }

    pub fn next_index(&self) -> usize {
        self.next_index
            .fetch_add(1, core::sync::atomic::Ordering::SeqCst)
    }

    fn load_program(&self, program: &ElfBytes<'_, AnyEndian>) -> Result<()> {
        let mut page_table = self.page_table.write();

        ElfLoader::new(program).load(|elf, phdr, vaddr, offst, len| {
            let mut perm = PagePerm::V | PagePerm::U;
            if phdr.p_flags & PF_R != 0 {
                perm |= PagePerm::R;
            }
            if phdr.p_flags & PF_W != 0 {
                perm |= PagePerm::W;
            }
            if phdr.p_flags & PF_X != 0 {
                perm |= PagePerm::X;
            }

            let paddr = if let Ok((phys_frame, old_perm)) = page_table.lookup(vaddr) {
                let paddr = phys_frame.addr();
                if !old_perm.contains(perm) {
                    page_table.map(vaddr, phys_frame, perm | old_perm)?;
                }
                paddr
            } else {
                let phys_frame = PhysFrame::alloc_in(self.allocator())?;
                let paddr = phys_frame.addr();
                page_table.map(vaddr, phys_frame, perm)?;
                paddr
            };

            if len != 0 {
                unsafe {
                    core::ptr::copy_nonoverlapping(
                        elf.segment_data(phdr)
                            .map_err(|_| InternalError::ElfParseError)?
                            .as_ptr(),
                        (paddr.to_virt().as_usize() + offst) as *mut u8,
                        len,
                    );
                }
            }

            Ok(())
        })?;

        hal!().cache().sync_all();
        hal!().vm().sync_all();

        Ok(())
    }
}

impl Drop for Partition {
    fn drop(&mut self) {
        PARTITIONS.write().remove(&self.identifier);
    }
}

impl PartitionProcessRegistry {
    const fn new() -> Self {
        Self {
            registry: BTreeMap::new(),
            names: BTreeMap::new(),
        }
    }

    fn insert(&mut self, process: Arc<Process>) {
        let identifier = process.identifier();
        let name = process.name();
        self.registry.insert(identifier, process.clone());
        self.names.insert(name, identifier);
    }
}

impl PartitionMemory {
    fn new(size: usize) -> Self {
        Self {
            size,
            free: Mutex::new(size),
        }
    }
}

unsafe impl Allocator for PartitionMemoryAllocator {
    fn allocate(
        &self,
        layout: core::alloc::Layout,
    ) -> core::prelude::v1::Result<core::ptr::NonNull<[u8]>, core::alloc::AllocError> {
        let partition = Partition::find(self.partition_id).ok_or(core::alloc::AllocError)?;
        let mut free = partition.memory.free.lock();
        if *free > layout.size() {
            *free -= layout.size();
            Global.allocate(layout)
        } else {
            Err(core::alloc::AllocError)
        }
    }

    unsafe fn deallocate(&self, ptr: core::ptr::NonNull<u8>, layout: core::alloc::Layout) {
        if let Some(partition) = Partition::find(self.partition_id) {
            let mut free = partition.memory.free.lock();
            *free += layout.size();
        }
        Global.deallocate(ptr, layout)
    }
}

impl PartitionConfig {
    pub fn new(
        name: ApexName,
        program: String,
        memory: usize,
        period: ApexSystemTime,
        duration: ApexSystemTime,
        num_cores: NumCores,
    ) -> Self {
        Self {
            name,
            program,
            memory,
            period,
            duration,
            num_cores,
        }
    }
}
