pub mod sched;

use core::sync::atomic::{AtomicUsize, Ordering};

use alloc::{
    string::{String, ToString},
    sync::Arc,
};
use spin::RwLock;

use crate::{
    arch::{self, mm::virt::PagePerm, task::SwitchInfo},
    conf,
    error::Result,
    mm::{
        phys,
        virt::{PageTable, VirtAddr, KERN_PAGE_TABLE},
    },
};

pub type TaskId = u64;

pub struct Task {
    ident: TaskId,
    name: String,
    priority: usize,
    switch_info: RwLock<SwitchInfo>,
    addrspace: Arc<RwLock<PageTable>>,
}

impl Task {
    pub fn create(name: &str, priority: usize, entry: usize) -> Result<Task> {
        let ident = Self::new_id();
        let name = name.to_string();
        let switch_info = RwLock::new(SwitchInfo::default());
        let addrspace = Arc::new(RwLock::new(PageTable::clone_from(
            &*KERN_PAGE_TABLE.read(),
        )?));
        let mut task = Self {
            ident,
            name,
            priority,
            switch_info,
            addrspace,
        };

        task.setup_vm(arch::layout::KTASK_STACK_TOP)?;

        task.switch_info
            .write()
            .entry(entry)
            .stack_top(arch::layout::KTASK_STACK_TOP)
            .page_table(&*task.addrspace.read());

        Ok(task)
    }

    pub fn with_page_table(&self, f: impl FnOnce(&mut PageTable)) {
        f(&mut self.addrspace.write());
    }

    pub fn get_ident(&self) -> TaskId {
        self.ident
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_priority(&self) -> usize {
        self.priority
    }

    fn new_id() -> TaskId {
        static ID: AtomicUsize = AtomicUsize::new(1);
        ID.fetch_add(1, Ordering::SeqCst).try_into().unwrap()
    }

    fn setup_vm(&mut self, stack_top: usize) -> Result<()> {
        let mut page_table = self.addrspace.write();
        for i in (0..conf::KSTACK_SIZE).step_by(conf::PAGE_SIZE) {
            let virt_addr = VirtAddr::new(stack_top - i - conf::PAGE_SIZE);
            let frame = phys::PhysFrame::alloc()?;
            page_table.map(virt_addr, frame, PagePerm::R | PagePerm::W)?;
        }
        Ok(())
    }
}
