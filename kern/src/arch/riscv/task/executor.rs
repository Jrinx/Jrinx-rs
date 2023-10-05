use crate::mm::virt::VirtAddr;

pub fn launch(address: VirtAddr, stack_top: VirtAddr) -> ! {
    unsafe {
        core::arch::asm!(
            "mv sp, {STACK_TOP}",
            "mv a0, {EXECUTOR}",
            "call {EXECUTOR_ENTRY}",
            STACK_TOP = in(reg) stack_top.as_usize(),
            EXECUTOR = in(reg) address.as_usize(),
            EXECUTOR_ENTRY = sym crate::task::executor::start,
            options(noreturn),
        );
    }
}
