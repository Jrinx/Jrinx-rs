pub(super) mod breakpoint {
    use crate::{arch, test::test_define, trap::breakpoint};

    test_define!("trap::breakpoint" => test);
    fn test() {
        for i in 0..10 {
            assert_eq!(breakpoint::count(), i);
            arch::breakpoint();
            assert_eq!(breakpoint::count(), i + 1);
        }
    }
}

pub(super) mod page_fault {
    use crate::{
        arch::{self, mm::virt::PagePerm, AbstractContext},
        mm::{
            phys,
            virt::{VirtAddr, KERN_PAGE_TABLE},
        },
        test::test_define,
        trap::TrapReason,
    };

    const USER_TEXT: usize = 0x10000;

    test_define!("trap::page_fault" => test);
    fn test() {
        let mut ctx = arch::trap::Context::default();
        ctx.setup_user(USER_TEXT, 0);
        ctx.run();
        assert_eq!(
            ctx.trap_reason(),
            TrapReason::PageFault {
                addr: VirtAddr::new(USER_TEXT),
                perm: PagePerm::X,
            }
        );

        let frame = phys::PhysFrame::alloc().unwrap();
        KERN_PAGE_TABLE
            .write()
            .map(
                VirtAddr::new(USER_TEXT),
                frame,
                PagePerm::U | PagePerm::R | PagePerm::W | PagePerm::X,
            )
            .unwrap();
        arch::mm::virt::sync(0, VirtAddr::new(USER_TEXT));

        code_read_zero(VirtAddr::new(USER_TEXT));
        ctx.run();
        assert_eq!(
            ctx.trap_reason(),
            TrapReason::PageFault {
                addr: VirtAddr::new(0),
                perm: PagePerm::R,
            }
        );

        code_write_zero(VirtAddr::new(USER_TEXT));
        ctx.run();
        assert_eq!(
            ctx.trap_reason(),
            TrapReason::PageFault {
                addr: VirtAddr::new(0),
                perm: PagePerm::W,
            }
        );
    }

    #[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
    fn code_write_zero(addr: VirtAddr) {
        unsafe {
            *(addr.as_usize() as *mut u32) = *(check_addr_protection as usize as *const u32);
        }

        #[naked]
        unsafe extern "C" fn check_addr_protection() -> ! {
            core::arch::asm!("sw zero, 0(zero)", options(noreturn));
        }
    }

    #[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
    fn code_read_zero(addr: VirtAddr) {
        unsafe {
            *(addr.as_usize() as *mut u32) = *(check_addr_protection as usize as *const u32);
        }

        #[naked]
        unsafe extern "C" fn check_addr_protection() -> ! {
            core::arch::asm!("lw zero, 0(zero)", options(noreturn));
        }
    }
}
