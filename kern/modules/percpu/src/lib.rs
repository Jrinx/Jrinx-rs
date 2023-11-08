#![no_std]
#![feature(allocator_api)]

extern crate alloc;

use core::alloc::{Allocator, Layout};

use alloc::alloc::Global;
use jrinx_config::PAGE_SIZE;
use jrinx_layout::{_epercpu, _spercpu};
use spin::{Lazy, Once};

mod arch;
pub use arch::*;

pub use jrinx_percpu_macro::*;

static GLOBAL_AREA_BASE: Once<usize> = Once::new();
static LOCAL_AREA_SIZE: Lazy<usize> = Lazy::new(|| _epercpu() - _spercpu());

pub fn global_area_base() -> usize {
    *GLOBAL_AREA_BASE.get().unwrap()
}

pub fn local_area_size() -> usize {
    *LOCAL_AREA_SIZE
}

pub fn local_area_base(cpu_id: usize) -> usize {
    global_area_base() + local_area_size() * cpu_id
}

pub fn init(nproc: usize) {
    let total_size = local_area_size() * nproc;
    let layout = Layout::from_size_align(total_size, PAGE_SIZE).unwrap();
    GLOBAL_AREA_BASE.call_once(|| {
        Global
            .allocate_zeroed(layout)
            .unwrap()
            .cast::<usize>()
            .as_ptr() as usize
    });

    let origin = _spercpu() as *const u8;
    for i in 0..nproc {
        let target = local_area_base(i) as *mut u8;
        unsafe {
            core::ptr::copy_nonoverlapping(origin, target, local_area_size());
        }
    }
}
