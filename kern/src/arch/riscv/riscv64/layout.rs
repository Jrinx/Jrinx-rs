use alloc::{vec, vec::Vec};

use crate::mm::virt::VirtAddr;

pub fn get_prot_addrs() -> Vec<(VirtAddr, usize)> {
    const BASE_ADDR: usize = 0x80000000;
    extern "C" {
        fn _end();
    }
    vec![(VirtAddr::new(BASE_ADDR), _end as usize - BASE_ADDR)]
}
