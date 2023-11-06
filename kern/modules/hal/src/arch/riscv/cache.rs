use crate::Cache;

#[derive(Debug, Clone, Copy)]
pub struct CacheImpl;

pub(crate) static CACHE: CacheImpl = CacheImpl;

impl Cache for CacheImpl {
    fn sync_all(&self) {
        unsafe {
            core::arch::asm!("fence.i");
        }
    }
}
