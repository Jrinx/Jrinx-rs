use core::mem::offset_of;

use const_default::ConstDefault;
use jrinx_addr::VirtAddr;

#[derive(Debug, Default, ConstDefault, Clone, Copy)]
#[repr(C)]
pub struct SwitchContext {
    ra: usize,
    sp: usize,
    s0: usize,
    s1: usize,
    s2: usize,
    s3: usize,
    s4: usize,
    s5: usize,
    s6: usize,
    s7: usize,
    s8: usize,
    s9: usize,
    s10: usize,
    s11: usize,
    satp: usize,
}

impl SwitchContext {
    pub(crate) fn new_executor(entry: VirtAddr, stack_top: VirtAddr) -> Self {
        Self {
            ra: entry.as_usize(),
            sp: stack_top.as_usize(),
            satp: riscv::register::satp::read().bits(),
            ..Default::default()
        }
    }

    pub(crate) fn init_executor_addr(&mut self, executor: VirtAddr) {
        self.s0 = executor.as_usize();
    }

    pub(crate) const fn new_runtime() -> Self {
        ConstDefault::DEFAULT
    }
}

#[cfg(target_arch = "riscv32")]
core::arch::global_asm! {
    r"
    .macro ST_REG reg, offset, base
        sw \reg, \offset(\base)
    .endm
    .macro LD_REG reg, offset, base
        lw \reg, \offset(\base)
    .endm
    "
}

#[cfg(target_arch = "riscv64")]
core::arch::global_asm! {
    r"
    .macro ST_REG reg, offset, base
        sd \reg, \offset(\base)
    .endm
    .macro LD_REG reg, offset, base
        ld \reg, \offset(\base)
    .endm
    "
}

core::arch::global_asm! {
    r"
    .global switch_context
    switch_context:
        ST_REG ra, {RA_OFFSET}, a0
        ST_REG sp, {SP_OFFSET}, a0
        ST_REG s0, {S0_OFFSET}, a0
        ST_REG s1, {S1_OFFSET}, a0
        ST_REG s2, {S2_OFFSET}, a0
        ST_REG s3, {S3_OFFSET}, a0
        ST_REG s4, {S4_OFFSET}, a0
        ST_REG s5, {S5_OFFSET}, a0
        ST_REG s6, {S6_OFFSET}, a0
        ST_REG s7, {S7_OFFSET}, a0
        ST_REG s8, {S8_OFFSET}, a0
        ST_REG s9, {S9_OFFSET}, a0
        ST_REG s10, {S10_OFFSET}, a0
        ST_REG s11, {S11_OFFSET}, a0
        csrr s0, satp
        ST_REG s0, {SATP_OFFSET}, a0

        LD_REG s0, {SATP_OFFSET}, a1
        csrw satp, s0
        sfence.vma x0, x0
        LD_REG s11, {S11_OFFSET}, a1
        LD_REG s10, {S10_OFFSET}, a1
        LD_REG s9, {S9_OFFSET}, a1
        LD_REG s8, {S8_OFFSET}, a1
        LD_REG s7, {S7_OFFSET}, a1
        LD_REG s6, {S6_OFFSET}, a1
        LD_REG s5, {S5_OFFSET}, a1
        LD_REG s4, {S4_OFFSET}, a1
        LD_REG s3, {S3_OFFSET}, a1
        LD_REG s2, {S2_OFFSET}, a1
        LD_REG s1, {S1_OFFSET}, a1
        LD_REG s0, {S0_OFFSET}, a1
        LD_REG sp, {SP_OFFSET}, a1
        LD_REG ra, {RA_OFFSET}, a1

        ret
    ",
    RA_OFFSET = const offset_of!(SwitchContext, ra),
    SP_OFFSET = const offset_of!(SwitchContext, sp),
    S0_OFFSET = const offset_of!(SwitchContext, s0),
    S1_OFFSET = const offset_of!(SwitchContext, s1),
    S2_OFFSET = const offset_of!(SwitchContext, s2),
    S3_OFFSET = const offset_of!(SwitchContext, s3),
    S4_OFFSET = const offset_of!(SwitchContext, s4),
    S5_OFFSET = const offset_of!(SwitchContext, s5),
    S6_OFFSET = const offset_of!(SwitchContext, s6),
    S7_OFFSET = const offset_of!(SwitchContext, s7),
    S8_OFFSET = const offset_of!(SwitchContext, s8),
    S9_OFFSET = const offset_of!(SwitchContext, s9),
    S10_OFFSET = const offset_of!(SwitchContext, s10),
    S11_OFFSET = const offset_of!(SwitchContext, s11),
    SATP_OFFSET = const offset_of!(SwitchContext, satp),
}

core::arch::global_asm! {
    r"
    .global executor_launch
    executor_launch:
        mv a0, s0
        call {EXECUTOR_START}
    ",
    EXECUTOR_START = sym crate::executor::Executor::start,
}

extern "C" {
    pub fn executor_launch();

    #[link_name = "switch_context"]
    pub fn switch(old_ctx: usize, new_ctx: usize);
}
