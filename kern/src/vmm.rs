use jrinx_hal::{Hal, Vm};
use jrinx_paging::{common::PageTable, GenericPageTable};
use spin::{Lazy, RwLock};

pub static KERN_PAGE_TABLE: Lazy<RwLock<PageTable>> =
    Lazy::new(|| RwLock::new(PageTable::new().unwrap()));

pub(super) fn init() {
    hal!().vm().enable(KERN_PAGE_TABLE.read().addr());
}
