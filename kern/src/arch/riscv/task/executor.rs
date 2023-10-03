pub fn launch(address: usize, stack_top: usize) -> ! {
    unsafe {
        core::arch::asm!(
            "mv sp, {STACK_TOP}",
            "mv a0, {EXECUTOR}",
            "call {EXECUTOR_ENTRY}",
            STACK_TOP = in(reg) stack_top,
            EXECUTOR = in(reg) address,
            EXECUTOR_ENTRY = sym crate::task::executor::start,
            options(noreturn),
        );
    }
}
