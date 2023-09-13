use alloc::{vec, vec::Vec};

use crate::{conf, mm::virt::VirtAddr};

pub fn get_prot_addrs() -> Vec<(VirtAddr, usize)> {
    const BASE_ADDR: usize = 0x80000000;
    vec![(VirtAddr::new(BASE_ADDR), conf::layout::_end() - BASE_ADDR)]
}
