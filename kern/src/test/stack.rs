use alloc::{vec, vec::Vec};
use jrinx_addr::VirtAddr;
use jrinx_config::PAGE_SIZE;
use jrinx_error::InternalError;
use jrinx_stack_alloc::StackAllocator;
use jrinx_testdef::testdef;
use spin::Mutex;

#[testdef]
fn test() {
    static MAP: Mutex<Vec<VirtAddr>> = Mutex::new(Vec::new());
    static UNMAP: Mutex<Vec<VirtAddr>> = Mutex::new(Vec::new());

    let stack_allocator = StackAllocator::new(
        (VirtAddr::new(0x1000), 4 * PAGE_SIZE),
        PAGE_SIZE,
        |addr| {
            MAP.lock().push(addr);
            Ok(())
        },
        |addr| {
            UNMAP.lock().push(addr);
            Ok(())
        },
    );

    let page_1 = stack_allocator.allocate(PAGE_SIZE);
    assert!(page_1.is_ok());

    let page_2 = stack_allocator.allocate(PAGE_SIZE);
    assert!(page_2.is_ok());

    assert!(stack_allocator
        .allocate(PAGE_SIZE)
        .is_err_and(|err| matches!(err, InternalError::NotEnoughMem)));

    assert!(stack_allocator.deallocate(page_1.unwrap()).is_ok());

    let page_3 = stack_allocator.allocate(PAGE_SIZE);
    assert!(page_3.is_ok());

    drop(stack_allocator);

    assert_eq!(
        *MAP.lock(),
        vec![
            VirtAddr::new(0x1000 + 2 * PAGE_SIZE),
            VirtAddr::new(0x1000 + 4 * PAGE_SIZE),
            VirtAddr::new(0x1000 + 2 * PAGE_SIZE),
        ]
    );

    assert_eq!(UNMAP.lock()[0], VirtAddr::new(0x1000 + 2 * PAGE_SIZE));
    assert!(UNMAP.lock()[1..].len() == 2);
    assert!(UNMAP.lock()[1..].iter().min() == Some(&VirtAddr::new(0x1000 + 2 * PAGE_SIZE)));
    assert!(UNMAP.lock()[1..].iter().max() == Some(&VirtAddr::new(0x1000 + 4 * PAGE_SIZE)));
}
