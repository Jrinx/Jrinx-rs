use alloc::{vec, vec::Vec};

use crate::{conf, mm::virt::VirtAddr};

#[cfg(target_arch = "riscv32")]
pub const KTASK_STACK_TOP: usize = 0xF0000000;

#[cfg(target_arch = "riscv64")]
pub const KTASK_STACK_TOP: usize = 0xFFFFFFF000000000;

pub fn get_prot_addrs() -> Vec<(VirtAddr, usize)> {
    const BASE_ADDR: usize = 0x80000000;
    vec![(VirtAddr::new(BASE_ADDR), conf::layout::_end() - BASE_ADDR)]
}
