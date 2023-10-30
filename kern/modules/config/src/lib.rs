#![no_std]

pub const PAGE_SIZE: usize = 4096;
pub const KSTACK_SIZE: usize = PAGE_SIZE * 8;

pub const HEAP_ORDER: usize = 32;
pub const KHEAP_SIZE: usize = PAGE_SIZE;

#[cfg(target_pointer_width = "32")]
pub const EXECUTOR_STACK_RANGE: (usize, usize) = (0xE0000000, 0xF0000000);

#[cfg(target_pointer_width = "64")]
pub const EXECUTOR_STACK_RANGE: (usize, usize) = (0xFFFFFFE000000000, 0xFFFFFFF000000000);
pub const EXECUTOR_STACK_SIZE: usize = PAGE_SIZE * 1024;
