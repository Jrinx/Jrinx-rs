#![no_std]

mod arch;
pub use arch::*;

pub struct RemapMemRegion {
    pub virt_addr: usize,
    pub phys_addr: usize,
    pub len: usize,
}

pub struct VirtMemRegion {
    pub addr: usize,
    pub len: usize,
}

pub const PAGE_SIZE: usize = 4096;
pub const KSTACK_SIZE: usize = PAGE_SIZE * 8;

pub const HEAP_ORDER: usize = 32;
pub const KHEAP_SIZE: usize = PAGE_SIZE * 8;

pub const EXECUTOR_STACK_SIZE: usize = PAGE_SIZE * 1024;
