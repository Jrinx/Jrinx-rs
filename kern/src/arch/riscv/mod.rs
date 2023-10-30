use jrinx_error::HaltReason;

pub mod cpu;
pub mod cpus;
pub mod earlycon;
pub mod mm;
pub mod task;
pub mod trap;

#[naked]
#[no_mangle]
#[link_section = ".boot"]
unsafe extern "C" fn _start() -> ! {
    extern "C" {
        static _estack: usize;
    }
    core::arch::asm!(
        "la sp, {INIT_STACK_TOP}",
        "mv tp, a0",
        "call {MAIN}",
        INIT_STACK_TOP = sym _estack,
        MAIN = sym crate::cold_init,
        options(noreturn),
    );
}

pub fn init() {
    unsafe {
        riscv::register::sstatus::set_sum();
        riscv::register::sie::set_sext();
        riscv::register::sie::set_stimer();
        riscv::register::sie::set_ssoft();
    }
    trap::init();
}

pub fn halt(reason: HaltReason) -> ! {
    let _ = sbi::system_reset::system_reset(
        sbi::system_reset::ResetType::WarmReboot,
        match reason {
            HaltReason::NormalExit => sbi::system_reset::ResetReason::NoReason,
            _ => sbi::system_reset::ResetReason::SystemFailure,
        },
    );
    unreachable!();
}

pub fn wait_for_interrupt() {
    unsafe {
        riscv::asm::wfi();
    }
}

pub fn breakpoint() {
    unsafe {
        riscv::asm::ebreak();
    }
}

pub fn int_is_enabled() -> bool {
    riscv::register::sstatus::read().sie()
}

pub fn int_enable() {
    unsafe {
        riscv::register::sstatus::set_sie();
    }
}

pub fn int_disable() {
    unsafe {
        riscv::register::sstatus::clear_sie();
    }
}
