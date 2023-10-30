#![no_std]

pub const PAGE_SIZE: usize = 4096;
pub const KSTACK_SIZE: usize = PAGE_SIZE * 8;

pub const HEAP_ORDER: usize = 32;
pub const KHEAP_SIZE: usize = PAGE_SIZE;
