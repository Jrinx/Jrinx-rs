use cfg_if::cfg_if;

use crate::{RemapMemRegion, VirtMemRegion};

pub const PHYS_MEM_BASE: usize = 0x8000_0000;

cfg_if! {
    if #[cfg(target_arch = "riscv32")] {
        pub const PHYS_MEM_LIMIT: usize = 0xC000_0000;
        pub const REMAP_HUGE_PAGE_SIZE: usize = 1024 * crate::PAGE_SIZE;
        pub const REMAP_MEM_OFFSET: usize = 0x0000_0000;
        pub const REMAP_MEM_REGIONS: &[RemapMemRegion] = &[
            RemapMemRegion {
                virt_addr: PHYS_MEM_BASE,
                phys_addr: PHYS_MEM_BASE,
                len: PHYS_MEM_LIMIT - PHYS_MEM_BASE,
            },
        ];
        pub const EXECUTOR_STACK_REGION: VirtMemRegion = VirtMemRegion {
            addr: 0xE000_0000,
            len: 0xF000_0000 - 0xE000_0000,
        };
        pub const UPROG_STACK_REGION: VirtMemRegion = VirtMemRegion {
            addr: 0x5000_0000,
            len: 0x7000_0000 - 0x5000_0000,
        };
    } else if #[cfg(target_arch = "riscv64")] {
        pub const PHYS_MEM_LIMIT: usize = 0x0000_0020_0000_0000;
        pub const REMAP_HUGE_PAGE_SIZE: usize = 512 * 512 * crate::PAGE_SIZE;
        pub const REMAP_MEM_OFFSET: usize = 0xFFFF_FFC0_0000_0000;
        pub const REMAP_MEM_REGIONS: &[RemapMemRegion] = &[
            RemapMemRegion {
                virt_addr: PHYS_MEM_BASE,
                phys_addr: PHYS_MEM_BASE,
                len: REMAP_HUGE_PAGE_SIZE,
            },
            RemapMemRegion {
                virt_addr: PHYS_MEM_BASE + REMAP_MEM_OFFSET,
                phys_addr: PHYS_MEM_BASE,
                len: PHYS_MEM_LIMIT - PHYS_MEM_BASE,
            },
        ];
        pub const EXECUTOR_STACK_REGION: VirtMemRegion = VirtMemRegion {
            addr: 0xFFFF_FFE0_0000_0000,
            len: 0xFFFF_FFFF_0000_0000 - 0xFFFF_FFE0_0000_0000,
        };
        pub const UPROG_STACK_REGION: VirtMemRegion = VirtMemRegion {
            addr: 0x0000_0020_0000_0000,
            len: 0x0000_0030_0000_0000 - 0x0000_0020_0000_0000,
        };
    } else {
        compile_error!("unsupported target_arch");
    }
}
