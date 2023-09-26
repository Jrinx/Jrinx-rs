use core::mem::{offset_of, size_of};

use super::SwitchInfo;

core::arch::global_asm! {
    r"
    .equ XLENB, {XLENB}
    .equ SWI_OFFS_RA, {SWI_OFFS_RA}
    .equ SWI_OFFS_SP, {SWI_OFFS_SP}
    .equ SWI_OFFS_GP, {SWI_OFFS_GP}
    .equ SWI_OFFS_S0, {SWI_OFFS_S0}
    .equ SWI_OFFS_S1, {SWI_OFFS_S1}
    .equ SWI_OFFS_S2, {SWI_OFFS_S2}
    .equ SWI_OFFS_S3, {SWI_OFFS_S3}
    .equ SWI_OFFS_S4, {SWI_OFFS_S4}
    .equ SWI_OFFS_S5, {SWI_OFFS_S5}
    .equ SWI_OFFS_S6, {SWI_OFFS_S6}
    .equ SWI_OFFS_S7, {SWI_OFFS_S7}
    .equ SWI_OFFS_S8, {SWI_OFFS_S8}
    .equ SWI_OFFS_S9, {SWI_OFFS_S9}
    .equ SWI_OFFS_S10, {SWI_OFFS_S10}
    .equ SWI_OFFS_S11, {SWI_OFFS_S11}
    .equ SWI_OFFS_SATP, {SWI_OFFS_SATP}
    ",
    XLENB = const size_of::<usize>(),
    SWI_OFFS_RA = const offset_of!(SwitchInfo, ra),
    SWI_OFFS_SP = const offset_of!(SwitchInfo, sp),
    SWI_OFFS_GP = const offset_of!(SwitchInfo, gp),
    SWI_OFFS_S0 = const offset_of!(SwitchInfo, s0),
    SWI_OFFS_S1 = const offset_of!(SwitchInfo, s1),
    SWI_OFFS_S2 = const offset_of!(SwitchInfo, s2),
    SWI_OFFS_S3 = const offset_of!(SwitchInfo, s3),
    SWI_OFFS_S4 = const offset_of!(SwitchInfo, s4),
    SWI_OFFS_S5 = const offset_of!(SwitchInfo, s5),
    SWI_OFFS_S6 = const offset_of!(SwitchInfo, s6),
    SWI_OFFS_S7 = const offset_of!(SwitchInfo, s7),
    SWI_OFFS_S8 = const offset_of!(SwitchInfo, s8),
    SWI_OFFS_S9 = const offset_of!(SwitchInfo, s9),
    SWI_OFFS_S10 = const offset_of!(SwitchInfo, s10),
    SWI_OFFS_S11 = const offset_of!(SwitchInfo, s11),
    SWI_OFFS_SATP = const offset_of!(SwitchInfo, satp),
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
    .global task_switch
    task_switch:
        ST_REG ra, SWI_OFFS_RA, a1
        ST_REG sp, SWI_OFFS_SP, a1
        ST_REG gp, SWI_OFFS_GP, a1
        ST_REG s0, SWI_OFFS_S0, a1
        ST_REG s1, SWI_OFFS_S1, a1
        ST_REG s2, SWI_OFFS_S2, a1
        ST_REG s3, SWI_OFFS_S3, a1
        ST_REG s4, SWI_OFFS_S4, a1
        ST_REG s5, SWI_OFFS_S5, a1
        ST_REG s6, SWI_OFFS_S6, a1
        ST_REG s7, SWI_OFFS_S7, a1
        ST_REG s8, SWI_OFFS_S8, a1
        ST_REG s9, SWI_OFFS_S9, a1
        ST_REG s10, SWI_OFFS_S10, a1
        ST_REG s11, SWI_OFFS_S11, a1

        csrr t0, satp
        ST_REG t0, SWI_OFFS_SATP, a1

    .global task_continue
    task_continue:
        LD_REG t0, SWI_OFFS_SATP, a0
        csrw satp, t0
        sfence.vma x0, x0

        LD_REG s11, SWI_OFFS_S11, a0
        LD_REG s10, SWI_OFFS_S10, a0
        LD_REG s9, SWI_OFFS_S9, a0
        LD_REG s8, SWI_OFFS_S8, a0
        LD_REG s7, SWI_OFFS_S7, a0
        LD_REG s6, SWI_OFFS_S6, a0
        LD_REG s5, SWI_OFFS_S5, a0
        LD_REG s4, SWI_OFFS_S4, a0
        LD_REG s3, SWI_OFFS_S3, a0
        LD_REG s2, SWI_OFFS_S2, a0
        LD_REG s1, SWI_OFFS_S1, a0
        LD_REG s0, SWI_OFFS_S0, a0
        LD_REG gp, SWI_OFFS_GP, a0
        LD_REG sp, SWI_OFFS_SP, a0
        LD_REG ra, SWI_OFFS_RA, a0

        ret
    "
}

extern "C" {
    pub fn task_switch(new: *mut SwitchInfo, old: *mut SwitchInfo);
    pub fn task_continue(info: *mut SwitchInfo) -> !;
}
