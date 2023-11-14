use crate::Cache;

#[derive(Debug, Clone, Copy)]
pub(crate) struct CacheImpl;

impl Cache for CacheImpl {
    fn sync_all(&self) {
        unsafe {
            core::arch::asm!("fence.i");
        }
    }
}
