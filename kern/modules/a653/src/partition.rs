use alloc::{
    alloc::Global,
    boxed::Box,
    collections::{BTreeMap, VecDeque},
    sync::{Arc, Weak},
    vec::Vec,
};
use core::{alloc::Allocator, ops::Deref, sync::atomic::AtomicUsize};

use elf::{
    abi::{PF_R, PF_W, PF_X},
    endian::AnyEndian,
    ElfBytes,
};
use jrinx_addr::VirtAddr;
use jrinx_apex::*;
use jrinx_error::{InternalError, Result};
use jrinx_hal::{hal, Cache, Hal, Vm};
use jrinx_loader::ElfLoader;
use jrinx_multitask::inspector::Inspector;
use jrinx_paging::{common::PageTable, GenericPagePerm, GenericPageTable, PagePerm};
use jrinx_phys_frame::PhysFrame;
use jrinx_serial_id_macro::SerialId;
use jrinx_stack_alloc::StackAllocator;
use jrinx_vmm::KERN_PAGE_TABLE;
use spin::{Mutex, RwLock, RwLockReadGuard};

use crate::{
    process::{Process, ProcessId},
    A653Entry,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, SerialId)]
pub struct PartitionId(ApexPartitionId);

pub struct Partition {
    kernel: bool,
    identifier: PartitionId,
    name: ApexName,
    memory: PartitionMemory,
    page_table: RwLock<PageTable>,
    pre_start_hooks: RwLock<VecDeque<Box<dyn FnOnce() + Send + Sync>>>,
    process_registry: RwLock<PartitionProcessRegistry>,
    stack_allocator: StackAllocator,
    next_index: AtomicUsize,
    entry: A653Entry,
    period: ApexSystemTime,
    duration: ApexSystemTime,
    lock_level: RwLock<ApexLockLevel>,
    operating_mode: RwLock<ApexOperatingMode>,
    start_condition: ApexStartCondition,
    num_assigned_cores: ApexNumCores,
    assigned_cores: RwLock<Vec<ApexProcessorCoreId>>,
}

struct PartitionMemory {
    size: usize,
    free: Mutex<usize>,
}

struct PartitionProcessRegistry {
    registry: BTreeMap<ProcessId, Arc<Process>>,
    names: BTreeMap<ApexName, ProcessId>,
}

#[derive(Debug, Clone, Copy)]
pub struct PartitionMemoryAllocator {
    partition_id: PartitionId,
}

pub struct PartitionConfig<'a> {
    pub name: ApexName,
    pub memory: usize,
    pub period: ApexSystemTime,
    pub duration: ApexSystemTime,
    pub num_cores: ApexNumCores,
    pub partition_type: PartitionTypeConfig<'a>,
}

pub enum PartitionTypeConfig<'a> {
    Kern, // TODO
    User(ElfBytes<'a, AnyEndian>),
}

static PARTITIONS: RwLock<BTreeMap<PartitionId, Weak<Partition>>> = RwLock::new(BTreeMap::new());

impl Partition {
    pub fn new(config: &PartitionConfig) -> Result<Arc<Self>> {
        let page_table = PageTable::new_from(&KERN_PAGE_TABLE.read())?;
        let partition_id = PartitionId::new();

        let stack_allocator = StackAllocator::new(
            (
                VirtAddr::new(jrinx_config::UPROG_STACK_REGION.addr),
                jrinx_config::UPROG_STACK_REGION.len,
            ),
            jrinx_config::PAGE_SIZE,
            move |addr| {
                let partition = Partition::find_by_id(partition_id).unwrap();
                partition.page_table.write().map(
                    addr,
                    PhysFrame::alloc_in(partition.allocator())?,
                    PagePerm::U | PagePerm::R | PagePerm::W,
                )?;
                Ok(())
            },
            move |addr| {
                if let Some(partition) = Partition::find_by_id(partition_id) {
                    partition.page_table.write().unmap(addr)?;
                }
                Ok(())
            },
        );

        let partition = Arc::new(Self {
            kernel: matches!(config.partition_type, PartitionTypeConfig::Kern),
            identifier: partition_id,
            name: config.name,
            memory: PartitionMemory::new(config.memory),
            page_table: RwLock::new(page_table),
            pre_start_hooks: RwLock::new(VecDeque::new()),
            process_registry: RwLock::new(PartitionProcessRegistry::new()),
            stack_allocator,
            next_index: AtomicUsize::new(0),
            period: config.period,
            duration: config.duration,
            lock_level: RwLock::new(APEX_LOCK_LEVEL_MIN),
            operating_mode: RwLock::new(ApexOperatingMode::Idle),
            start_condition: ApexStartCondition::NormalStart,
            num_assigned_cores: config.num_cores,
            assigned_cores: RwLock::new(Vec::new()),
            entry: match &config.partition_type {
                PartitionTypeConfig::Kern => todo!(),
                PartitionTypeConfig::User(program) => A653Entry::User(program.ehdr.e_entry as _),
            },
        });

        PARTITIONS
            .write()
            .insert(partition.identifier, Arc::downgrade(&partition));

        if let PartitionTypeConfig::User(program) = &config.partition_type {
            partition.load_program(program)?;
        }

        Ok(partition)
    }

    pub fn current() -> Option<Arc<Self>> {
        Inspector::with_current(|is| is.ext().deref().downcast_ref().cloned()).ok()?
    }

    pub fn find_by_id(id: PartitionId) -> Option<Arc<Self>> {
        PARTITIONS.read().get(&id)?.upgrade()
    }

    pub fn find_by_name(name: &ApexName) -> Option<Arc<Self>> {
        PARTITIONS
            .read()
            .values()
            .find_map(|p| p.upgrade().filter(|p| p.name == *name))
    }

    pub fn kernel(&self) -> bool {
        self.kernel
    }

    pub fn identifier(&self) -> PartitionId {
        self.identifier
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

    pub fn entry(&self) -> A653Entry {
        self.entry
    }

    pub fn period(&self) -> ApexSystemTime {
        self.period
    }

    pub fn duration(&self) -> ApexSystemTime {
        self.duration
    }

    pub fn operating_mode(&self) -> ApexOperatingMode {
        *self.operating_mode.read()
    }

    pub fn set_operating_mode(&self, mode: ApexOperatingMode) {
        *self.operating_mode.write() = mode;
    }

    pub fn lock_level(&self) -> ApexLockLevel {
        *self.lock_level.read()
    }

    pub fn set_lock_level(&self, level: ApexLockLevel) {
        *self.lock_level.write() = level;
    }

    pub fn assigned_cores(&self) -> Vec<ApexProcessorCoreId> {
        self.assigned_cores.read().clone()
    }

    pub fn assign_core(&self, core_id: ApexProcessorCoreId) {
        self.assigned_cores.write().push(core_id);
    }

    pub fn allocator(&self) -> PartitionMemoryAllocator {
        PartitionMemoryAllocator {
            partition_id: self.identifier,
        }
    }

    pub fn add_pre_start_hook(&self, hook: impl FnOnce() + Send + Sync + 'static) {
        self.pre_start_hooks.write().push_back(Box::new(hook));
    }

    pub fn run_pre_start_hooks(&self) {
        let mut hooks = self.pre_start_hooks.write();
        while let Some(hook) = hooks.pop_front() {
            hook();
        }
    }

    pub fn pt_read(&self) -> RwLockReadGuard<'_, PageTable> {
        self.page_table.read()
    }

    pub fn pt_sync(&self) {
        let kern_page_table = KERN_PAGE_TABLE.read();
        let curr_page_table = self.page_table.upgradeable_read();

        if kern_page_table.generation() != curr_page_table.generation() {
            let mut page_table = curr_page_table.upgrade();
            page_table.sync_with(&kern_page_table);
            page_table.sync_generation(&kern_page_table);
            hal!().vm().sync_all();
        }
    }

    pub fn status(&self) -> ApexPartitionStatus {
        ApexPartitionStatus {
            period: self.period,
            duration: self.duration,
            identifier: self.identifier.0,
            lock_level: self.lock_level(),
            operating_mode: self.operating_mode(),
            start_condition: self.start_condition,
            num_assigned_cores: self.num_assigned_cores,
        }
    }

    pub fn gen_inspector(self: &Arc<Self>) -> Result<Inspector> {
        Ok(Inspector::new_with_ext(self.clone()))
    }

    pub(crate) fn allocate_stack(&self, stack_size: usize) -> Result<VirtAddr> {
        self.stack_allocator.allocate(stack_size)
    }

    pub(crate) fn deallocate_stack(&self, stack_top: VirtAddr) -> Result<()> {
        self.stack_allocator.deallocate(stack_top)
    }

    pub(crate) fn next_index(&self) -> usize {
        self.next_index
            .fetch_add(1, core::sync::atomic::Ordering::SeqCst)
    }

    pub(crate) fn register_process(&self, process: Arc<Process>) {
        self.process_registry.write().insert(process);
    }

    pub(crate) fn find_process_by_id(&self, identifier: ProcessId) -> Option<Arc<Process>> {
        self.process_registry
            .read()
            .registry
            .get(&identifier)
            .cloned()
    }

    pub(crate) fn find_process_by_name(&self, name: &ApexProcessName) -> Option<Arc<Process>> {
        self.process_registry
            .read()
            .names
            .get(name)
            .and_then(|id| self.find_process_by_id(*id))
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
                            .as_ptr()
                            .add(offst),
                        (paddr.to_virt().as_usize() + (vaddr - vaddr.align_page_down())) as *mut u8,
                        len,
                    );
                }
            }

            Ok(())
        })?;

        hal!().cache().sync_all();

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
        let partition = Partition::find_by_id(self.partition_id).ok_or(core::alloc::AllocError)?;
        let mut free = partition.memory.free.lock();
        if *free > layout.size() {
            *free -= layout.size();
            Global.allocate(layout)
        } else {
            Err(core::alloc::AllocError)
        }
    }

    unsafe fn deallocate(&self, ptr: core::ptr::NonNull<u8>, layout: core::alloc::Layout) {
        if let Some(partition) = Partition::find_by_id(self.partition_id) {
            *partition.memory.free.lock() += layout.size();
        }
        Global.deallocate(ptr, layout)
    }
}
