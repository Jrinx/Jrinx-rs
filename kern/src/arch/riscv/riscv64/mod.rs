use crate::conf;

#[naked]
#[no_mangle]
#[link_section = ".boot"]
unsafe extern "C" fn _start() -> ! {
    static mut INIT_STACK: [u8; conf::KSTACK_SIZE] = [0; conf::KSTACK_SIZE];
    core::arch::asm!(
        "la sp, {INIT_STACK_BOTTOM}",
        "li t0, {INIT_STACK_SIZE}",
        "add sp, sp, t0", // setup stack
        "mv tp, a0",
        "call {MAIN}",
        INIT_STACK_BOTTOM = sym INIT_STACK,
        INIT_STACK_SIZE = const conf::KSTACK_SIZE,
        MAIN = sym crate::main,
        options(noreturn),
    );
}
