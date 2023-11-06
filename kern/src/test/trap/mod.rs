pub(super) mod breakpoint {
    use jrinx_hal::Hal;
    use jrinx_testdef_macro::testdef;

    use crate::trap::breakpoint;

    #[testdef]
    fn test() {
        for i in 0..10 {
            assert_eq!(breakpoint::count(), i);
            hal!().breakpoint();
            assert_eq!(breakpoint::count(), i + 1);
        }
    }
}

pub(super) mod page_fault {
    use cfg_if::cfg_if;
    use jrinx_addr::VirtAddr;
    use jrinx_hal::{Cache, Hal, Vm};
    use jrinx_paging::{GenericPagePerm, GenericPageTable, PagePerm};
    use jrinx_phys_frame::PhysFrame;
    use jrinx_testdef_macro::testdef;

    use crate::{arch, trap::TrapReason, vmm::KERN_PAGE_TABLE};

    cfg_if! {
        if #[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))] {
            macro_rules! code_read_zero {
                ($addr:expr) => {
                    {
                        unsafe {
                            *($addr.as_usize() as *mut u32) = *(check_addr_protection as usize as *const u32);
                        }
                        hal!().cache().sync_all();
                        #[naked]
                        unsafe extern "C" fn check_addr_protection() -> ! {
                            core::arch::asm!("lw zero, 0(zero)", options(noreturn));
                        }
                    }
                };
            }

            macro_rules! code_write_zero {
                ($addr:expr) => {
                    {
                        unsafe {
                            *($addr.as_usize() as *mut u32) = *(check_addr_protection as usize as *const u32);
                        }
                        hal!().cache().sync_all();
                        #[naked]
                        unsafe extern "C" fn check_addr_protection() -> ! {
                            core::arch::asm!("sw zero, 0(zero)", options(noreturn));
                        }
                    }
                };
            }
        }
    }

    const USER_TEXT: usize = 0x10000;

    #[testdef]
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

        let frame = PhysFrame::alloc().unwrap();
        let addr = frame.addr();
        KERN_PAGE_TABLE
            .write()
            .map(
                VirtAddr::new(USER_TEXT),
                frame,
                PagePerm::U | PagePerm::R | PagePerm::X,
            )
            .unwrap();

        hal!().vm().sync_all();

        code_read_zero!(addr.to_virt());
        ctx.run();
        assert_eq!(
            ctx.trap_reason(),
            TrapReason::PageFault {
                addr: VirtAddr::new(0),
                perm: PagePerm::R,
            }
        );

        code_write_zero!(addr.to_virt());
        ctx.run();
        assert_eq!(
            ctx.trap_reason(),
            TrapReason::PageFault {
                addr: VirtAddr::new(0),
                perm: PagePerm::W,
            }
        );
    }
}

pub(super) mod syscall {
    use jrinx_addr::VirtAddr;
    use jrinx_hal::{Cache, Hal, Vm};
    use jrinx_paging::{GenericPagePerm, GenericPageTable, PagePerm};
    use jrinx_phys_frame::PhysFrame;
    use jrinx_testdef_macro::testdef;

    use crate::{arch, trap::TrapReason, vmm::KERN_PAGE_TABLE};

    #[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
    macro_rules! code_syscall_with_num {
        ($addr:expr, $num:literal) => {
            {
                unsafe {
                    *($addr.as_usize() as *mut u64) = *(syscall_with_num as *const u64);
                }
                hal!().cache().sync_all();
                #[naked]
                unsafe extern "C" fn syscall_with_num() -> ! {
                    core::arch::asm!("ori a7, zero, {}", "ecall", const $num, options(noreturn));
                }
            }
        };
    }

    const USER_TEXT: usize = 0x10000;

    #[testdef]
    fn test() {
        let mut ctx = arch::trap::Context::default();

        let frame = PhysFrame::alloc().unwrap();
        let addr = frame.addr();
        KERN_PAGE_TABLE
            .write()
            .map(
                VirtAddr::new(USER_TEXT),
                frame,
                PagePerm::U | PagePerm::R | PagePerm::X,
            )
            .unwrap();

        hal!().vm().sync_all();

        code_syscall_with_num!(addr.to_virt(), 32);
        ctx.setup_user(USER_TEXT, 0);
        ctx.run();
        assert_eq!(ctx.trap_reason(), TrapReason::SystemCall);
        assert_eq!(ctx.get_syscall_num(), 32);

        code_syscall_with_num!(addr.to_virt(), 64);
        ctx.setup_user(USER_TEXT, 0);
        ctx.run();
        assert_eq!(ctx.trap_reason(), TrapReason::SystemCall);
        assert_eq!(ctx.get_syscall_num(), 64);
    }
}
