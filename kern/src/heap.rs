use buddy_system_allocator::LockedHeap;

use crate::mm::virt::VirtAddr;

const HEAP_ORDER: usize = 32;
const HEAP_SIZE: usize = jrinx_config::PAGE_SIZE;

#[global_allocator]
static mut HEAP_ALLOCATOR: LockedHeap<HEAP_ORDER> = LockedHeap::new();

pub(super) fn init() {
    #[repr(C, align(4096))]
    struct HeapSpace([u8; HEAP_SIZE]);
    static mut HEAP_SPACE: HeapSpace = HeapSpace([0; HEAP_SIZE]);
    unsafe {
        HEAP_ALLOCATOR
            .lock()
            .init(HEAP_SPACE.0.as_ptr() as usize, HEAP_SIZE);
    };
}

pub fn enlarge(region: (VirtAddr, usize)) {
    unsafe {
        HEAP_ALLOCATOR
            .lock()
            .add_to_heap(region.0.as_usize(), region.0.as_usize() + region.1);
    }
}
