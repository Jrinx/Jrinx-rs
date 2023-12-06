use elf::{
    abi::{PF_R, PF_W, PF_X},
    endian::AnyEndian,
    ElfBytes,
};
use jrinx_hal::{Cache, Hal, Vm};
use jrinx_loader::ElfLoader;
use jrinx_paging::{GenericPagePerm, GenericPageTable, PagePerm};
use jrinx_phys_frame::PhysFrame;
use jrinx_vmm::KERN_PAGE_TABLE;

pub(super) mod breakpoint {
    use jrinx_hal::Hal;
    use jrinx_testdef::testdef;
    use jrinx_trap::breakpoint;

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
    use jrinx_addr::VirtAddr;
    use jrinx_paging::{GenericPagePerm, PagePerm};
    use jrinx_testdef::testdef;
    use jrinx_trap::{arch::Context, GenericContext, TrapReason};

    #[testdef]
    fn test() {
        let nullptr_reader = jrinx_uprog::find("test/nullptr-reader").unwrap();
        let nullptr_writer = jrinx_uprog::find("test/nullptr-writer").unwrap();

        let nullptr_reader_entry = nullptr_reader.ehdr.e_entry as usize;
        let nullptr_writer_entry = nullptr_writer.ehdr.e_entry as usize;

        let mut ctx = Context::default();

        ctx.user_setup(0, 0);
        ctx.disable_int();
        ctx.run();
        assert_eq!(
            ctx.trap_reason(),
            TrapReason::PageFault {
                addr: VirtAddr::new(0),
                perm: PagePerm::X,
            }
        );

        super::load_elf(nullptr_reader);
        ctx.user_setup(nullptr_reader_entry, 0);
        ctx.disable_int();
        ctx.run();
        assert_eq!(
            ctx.trap_reason(),
            TrapReason::PageFault {
                addr: VirtAddr::new(0),
                perm: PagePerm::R,
            }
        );

        super::load_elf(nullptr_writer);
        ctx.user_setup(nullptr_writer_entry, 0);
        ctx.disable_int();
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
    use jrinx_testdef::testdef;
    use jrinx_trap::{arch::Context, GenericContext, TrapReason};

    #[testdef]
    fn test() {
        let system_caller = jrinx_uprog::find("test/system-caller").unwrap();
        let system_caller_entry = system_caller.ehdr.e_entry as usize;

        super::load_elf(system_caller);

        let mut ctx = Context::default();

        ctx.user_setup(system_caller_entry, 0);
        ctx.disable_int();
        ctx.run();
        assert_eq!(ctx.trap_reason(), TrapReason::SystemCall);
        assert_eq!(ctx.syscall_num(), 0xC0DE);
    }
}

fn load_elf(elf: ElfBytes<'_, AnyEndian>) {
    ElfLoader::new(&elf)
        .load(|elf, phdr, vaddr, offst, len| {
            let mut perm = PagePerm::V | PagePerm::U;
            if phdr.p_flags & PF_R != 0 {
                perm |= PagePerm::R;
            }
            if phdr.p_flags & PF_W != 0 {
                perm |= PagePerm::W;
            }
            if phdr.p_flags & PF_X != 0 {
                perm |= PagePerm::X;
            }
            let mut page_table = KERN_PAGE_TABLE.write();

            let paddr = if let Ok((phys_frame, old_perm)) = page_table.lookup(vaddr) {
                let paddr = phys_frame.addr();
                if !old_perm.contains(perm) {
                    page_table.map(vaddr, phys_frame, perm | old_perm).unwrap();
                }
                paddr
            } else {
                let phys_frame = PhysFrame::alloc().unwrap();
                let paddr = phys_frame.addr();
                page_table.map(vaddr, phys_frame, perm).unwrap();
                paddr
            };
            if len != 0 {
                unsafe {
                    core::ptr::copy_nonoverlapping(
                        elf.segment_data(phdr).unwrap().as_ptr(),
                        (paddr.to_virt().as_usize() + offst) as *mut u8,
                        len,
                    );
                }
            }
            Ok(())
        })
        .unwrap();
    hal!().cache().sync_all();
    hal!().vm().sync_all();
}
