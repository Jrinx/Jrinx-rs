pub(super) mod phys {
    use core::mem::forget;

    use alloc::sync::Arc;

    use crate::{
        error::Result,
        mm::phys::{PhysAddr, PhysFrame},
    };

    pub fn test() {
        let (frame1, addr1) = alloc().unwrap();
        let (frame2, addr2) = alloc().unwrap();

        assert_ne!(addr1, addr2);

        let frame3 = frame1.clone();
        drop(frame1);
        drop(frame2);

        while let Ok((frame, _)) = alloc() {
            forget(frame);
        }
        drop(frame3);

        let (_, addr4) = alloc().unwrap();
        assert_eq!(addr1, addr4);
    }

    fn alloc() -> Result<(Arc<PhysFrame>, PhysAddr)> {
        let f = PhysFrame::alloc()?;
        let a = f.addr();
        Ok((f, a))
    }
}

pub(super) mod virt {
    use core::mem;

    use crate::{
        arch::{self, mm::virt::PagePerm},
        conf,
        mm::{
            phys::PhysFrame,
            virt::{VirtAddr, KERN_PAGE_TABLE},
        },
    };

    pub fn test() {
        let vaddr1 = VirtAddr::new(conf::PAGE_SIZE);
        let vaddr2 = VirtAddr::new(conf::PAGE_SIZE * 2);

        for test_seed in (1..=10).map(|i| i * 0xdeadbeef) {
            let frame = PhysFrame::alloc().unwrap();
            let paddr = frame.addr();

            let mut allocator = KERN_PAGE_TABLE.write();
            allocator
                .map(vaddr1, frame, PagePerm::G | PagePerm::W | PagePerm::R)
                .unwrap();

            let (frame, perm) = allocator.lookup(vaddr1).unwrap();
            assert_eq!(frame.addr(), paddr);
            allocator.map(vaddr2, frame, perm).unwrap();

            let (paddr1, perm1) = allocator.translate(vaddr1).unwrap();
            let (paddr2, perm2) = allocator.translate(vaddr2).unwrap();
            assert_eq!(paddr1, paddr2);
            assert_eq!(perm1.bits(), perm2.bits());

            arch::mm::virt::sync(0, vaddr1);
            arch::mm::virt::sync(0, vaddr2);

            let space = [
                vaddr1.as_usize() as *mut usize,
                paddr.as_usize() as *mut usize,
            ];

            for i in 0..conf::PAGE_SIZE / mem::size_of::<usize>() {
                let src = space[i % 2];
                let dst = space[1 - i % 2];
                write(src, i * test_seed);
                assert_eq!(read(dst), i * test_seed);
                write(src, !read(dst));
                assert_eq!(read(src), read(dst));
            }
        }

        let mut allocator = KERN_PAGE_TABLE.write();
        allocator.unmap(vaddr1).unwrap();
        allocator.unmap(vaddr2).unwrap();
    }

    fn write<T>(src: *mut T, val: T)
    where
        T: Clone + Copy,
    {
        unsafe { *src = val }
    }

    fn read<T>(dst: *const T) -> T
    where
        T: Clone + Copy,
    {
        unsafe { *dst.clone() }
    }
}
