#![no_std]

use buddy_system_allocator::LockedHeap;
use jrinx_addr::VirtAddr;

use jrinx_config::{HEAP_ORDER, KHEAP_SIZE};

#[global_allocator]
static mut HEAP_ALLOCATOR: LockedHeap<HEAP_ORDER> = LockedHeap::new();

pub fn init() {
    #[repr(C, align(4096))]
    struct HeapSpace([u8; KHEAP_SIZE]);
    static mut HEAP_SPACE: HeapSpace = HeapSpace([0; KHEAP_SIZE]);
    unsafe {
        HEAP_ALLOCATOR
            .lock()
            .init(HEAP_SPACE.0.as_ptr() as usize, KHEAP_SIZE);
    };
}

pub fn enlarge(region: (VirtAddr, usize)) {
    unsafe {
        HEAP_ALLOCATOR
            .lock()
            .add_to_heap(region.0.as_usize(), region.0.as_usize() + region.1);
    }
}
