use cfg_if::cfg_if;

use crate::{conf, mm::virt::PageTable, task::sched};

mod entry;

#[derive(Debug, Default, Clone, Copy)]
#[repr(C)]
pub struct SwitchInfo {
    ra: usize,
    sp: usize,
    gp: usize,
    s0: usize,
    s1: usize,
    s2: usize,
    s3: usize,
    s4: usize,
    s5: usize,
    s6: usize,
    s7: usize,
    s8: usize,
    s9: usize,
    s10: usize,
    s11: usize,
    satp: usize,
}

impl SwitchInfo {
    pub fn new() -> Self {
        extern "C" {
            fn task_entry();
        }
        Self {
            ra: task_entry as usize,
            ..Default::default()
        }
    }

    pub fn stack_top(&mut self, top: usize) -> &mut Self {
        self.sp = top;
        self
    }

    pub fn page_table(&mut self, page_table: &PageTable) -> &mut Self {
        cfg_if! {
            if #[cfg(feature = "pt_level_2")] {
                self.satp = (riscv::register::satp::Mode::Sv32 as usize) << 31 |
                    (page_table.addr().as_usize() / conf::PAGE_SIZE);
            } else if #[cfg(feature = "pt_level_3")] {
                self.satp = (riscv::register::satp::Mode::Sv39 as usize) << 60 |
                    (page_table.addr().as_usize() / conf::PAGE_SIZE);
            } else {
                unimplemented!();
            }
        }
        self
    }
}

#[inline]
pub fn switch(new: *mut SwitchInfo, old: *mut SwitchInfo) {
    extern "C" {
        fn task_switch(new: *mut SwitchInfo, old: *mut SwitchInfo);
    }
    unsafe {
        task_switch(new, old);
    }
}

#[inline]
pub fn resume(new: *mut SwitchInfo) -> ! {
    extern "C" {
        fn task_continue(info: *mut SwitchInfo) -> !;
    }
    unsafe {
        task_continue(new);
    }
}

extern "C" fn task_entry_wrapper(arg: usize, next: usize) -> ! {
    unsafe {
        core::arch::asm!(
            "mv a0, {ARG}",
            "jalr {NEXT}",
            ARG = in(reg) arg,
            NEXT = in(reg) next,
        );
    }
    sched::global_destroy();
}
