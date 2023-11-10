use super::{handle_kern_trap, Context};

use core::mem::{offset_of, size_of};

core::arch::global_asm! {
    r#".attribute arch, "rv64imafd""#,
    r"
    .equ XLENB, {XLENB}
    .equ CTX_SIZE, {CTX_SIZE}
    .equ CTX_OFFS_REG_ZERO, {CTX_OFFS_REG_ZERO}
    .equ CTX_OFFS_REG_RA, {CTX_OFFS_REG_RA}
    .equ CTX_OFFS_REG_SP, {CTX_OFFS_REG_SP}
    .equ CTX_OFFS_REG_GP, {CTX_OFFS_REG_GP}
    .equ CTX_OFFS_REG_TP, {CTX_OFFS_REG_TP}
    .equ CTX_OFFS_REG_T0, {CTX_OFFS_REG_T0}
    .equ CTX_OFFS_REG_T1, {CTX_OFFS_REG_T1}
    .equ CTX_OFFS_REG_T2, {CTX_OFFS_REG_T2}
    .equ CTX_OFFS_REG_S0, {CTX_OFFS_REG_S0}
    .equ CTX_OFFS_REG_S1, {CTX_OFFS_REG_S1}
    .equ CTX_OFFS_REG_A0, {CTX_OFFS_REG_A0}
    .equ CTX_OFFS_REG_A1, {CTX_OFFS_REG_A1}
    .equ CTX_OFFS_REG_A2, {CTX_OFFS_REG_A2}
    .equ CTX_OFFS_REG_A3, {CTX_OFFS_REG_A3}
    .equ CTX_OFFS_REG_A4, {CTX_OFFS_REG_A4}
    .equ CTX_OFFS_REG_A5, {CTX_OFFS_REG_A5}
    .equ CTX_OFFS_REG_A6, {CTX_OFFS_REG_A6}
    .equ CTX_OFFS_REG_A7, {CTX_OFFS_REG_A7}
    .equ CTX_OFFS_REG_S2, {CTX_OFFS_REG_S2}
    .equ CTX_OFFS_REG_S3, {CTX_OFFS_REG_S3}
    .equ CTX_OFFS_REG_S4, {CTX_OFFS_REG_S4}
    .equ CTX_OFFS_REG_S5, {CTX_OFFS_REG_S5}
    .equ CTX_OFFS_REG_S6, {CTX_OFFS_REG_S6}
    .equ CTX_OFFS_REG_S7, {CTX_OFFS_REG_S7}
    .equ CTX_OFFS_REG_S8, {CTX_OFFS_REG_S8}
    .equ CTX_OFFS_REG_S9, {CTX_OFFS_REG_S9}
    .equ CTX_OFFS_REG_S10, {CTX_OFFS_REG_S10}
    .equ CTX_OFFS_REG_S11, {CTX_OFFS_REG_S11}
    .equ CTX_OFFS_REG_T3, {CTX_OFFS_REG_T3}
    .equ CTX_OFFS_REG_T4, {CTX_OFFS_REG_T4}
    .equ CTX_OFFS_REG_T5, {CTX_OFFS_REG_T5}
    .equ CTX_OFFS_REG_T6, {CTX_OFFS_REG_T6}

    .equ CTX_OFFS_FREG_FT0, {CTX_OFFS_FREG_FT0}
    .equ CTX_OFFS_FREG_FT1, {CTX_OFFS_FREG_FT1}
    .equ CTX_OFFS_FREG_FT2, {CTX_OFFS_FREG_FT2}
    .equ CTX_OFFS_FREG_FT3, {CTX_OFFS_FREG_FT3}
    .equ CTX_OFFS_FREG_FT4, {CTX_OFFS_FREG_FT4}
    .equ CTX_OFFS_FREG_FT5, {CTX_OFFS_FREG_FT5}
    .equ CTX_OFFS_FREG_FT6, {CTX_OFFS_FREG_FT6}
    .equ CTX_OFFS_FREG_FT7, {CTX_OFFS_FREG_FT7}
    .equ CTX_OFFS_FREG_FS0, {CTX_OFFS_FREG_FS0}
    .equ CTX_OFFS_FREG_FS1, {CTX_OFFS_FREG_FS1}
    .equ CTX_OFFS_FREG_FA0, {CTX_OFFS_FREG_FA0}
    .equ CTX_OFFS_FREG_FA1, {CTX_OFFS_FREG_FA1}
    .equ CTX_OFFS_FREG_FA2, {CTX_OFFS_FREG_FA2}
    .equ CTX_OFFS_FREG_FA3, {CTX_OFFS_FREG_FA3}
    .equ CTX_OFFS_FREG_FA4, {CTX_OFFS_FREG_FA4}
    .equ CTX_OFFS_FREG_FA5, {CTX_OFFS_FREG_FA5}
    .equ CTX_OFFS_FREG_FA6, {CTX_OFFS_FREG_FA6}
    .equ CTX_OFFS_FREG_FA7, {CTX_OFFS_FREG_FA7}
    .equ CTX_OFFS_FREG_FS2, {CTX_OFFS_FREG_FS2}
    .equ CTX_OFFS_FREG_FS3, {CTX_OFFS_FREG_FS3}
    .equ CTX_OFFS_FREG_FS4, {CTX_OFFS_FREG_FS4}
    .equ CTX_OFFS_FREG_FS5, {CTX_OFFS_FREG_FS5}
    .equ CTX_OFFS_FREG_FS6, {CTX_OFFS_FREG_FS6}
    .equ CTX_OFFS_FREG_FS7, {CTX_OFFS_FREG_FS7}
    .equ CTX_OFFS_FREG_FS8, {CTX_OFFS_FREG_FS8}
    .equ CTX_OFFS_FREG_FS9, {CTX_OFFS_FREG_FS9}
    .equ CTX_OFFS_FREG_FS10, {CTX_OFFS_FREG_FS10}
    .equ CTX_OFFS_FREG_FS11, {CTX_OFFS_FREG_FS11}
    .equ CTX_OFFS_FREG_FT8, {CTX_OFFS_FREG_FT8}
    .equ CTX_OFFS_FREG_FT9, {CTX_OFFS_FREG_FT9}
    .equ CTX_OFFS_FREG_FT10, {CTX_OFFS_FREG_FT10}
    .equ CTX_OFFS_FREG_FT11, {CTX_OFFS_FREG_FT11}

    .equ CTX_OFFS_SSTATUS, {CTX_OFFS_SSTATUS}
    .equ CTX_OFFS_SCAUSE, {CTX_OFFS_SCAUSE}
    .equ CTX_OFFS_SIE, {CTX_OFFS_SIE}
    .equ CTX_OFFS_STVAL, {CTX_OFFS_STVAL}
    .equ CTX_OFFS_SEPC, {CTX_OFFS_SEPC}
    ",
    XLENB = const size_of::<usize>(),
    CTX_SIZE = const size_of::<Context>(),
    CTX_OFFS_REG_ZERO = const offset_of!(Context, regs.zero),
    CTX_OFFS_REG_RA = const offset_of!(Context, regs.ra),
    CTX_OFFS_REG_SP = const offset_of!(Context, regs.sp),
    CTX_OFFS_REG_GP = const offset_of!(Context, regs.gp),
    CTX_OFFS_REG_TP = const offset_of!(Context, regs.tp),
    CTX_OFFS_REG_T0 = const offset_of!(Context, regs.t0),
    CTX_OFFS_REG_T1 = const offset_of!(Context, regs.t1),
    CTX_OFFS_REG_T2 = const offset_of!(Context, regs.t2),
    CTX_OFFS_REG_S0 = const offset_of!(Context, regs.s0),
    CTX_OFFS_REG_S1 = const offset_of!(Context, regs.s1),
    CTX_OFFS_REG_A0 = const offset_of!(Context, regs.a0),
    CTX_OFFS_REG_A1 = const offset_of!(Context, regs.a1),
    CTX_OFFS_REG_A2 = const offset_of!(Context, regs.a2),
    CTX_OFFS_REG_A3 = const offset_of!(Context, regs.a3),
    CTX_OFFS_REG_A4 = const offset_of!(Context, regs.a4),
    CTX_OFFS_REG_A5 = const offset_of!(Context, regs.a5),
    CTX_OFFS_REG_A6 = const offset_of!(Context, regs.a6),
    CTX_OFFS_REG_A7 = const offset_of!(Context, regs.a7),
    CTX_OFFS_REG_S2 = const offset_of!(Context, regs.s2),
    CTX_OFFS_REG_S3 = const offset_of!(Context, regs.s3),
    CTX_OFFS_REG_S4 = const offset_of!(Context, regs.s4),
    CTX_OFFS_REG_S5 = const offset_of!(Context, regs.s5),
    CTX_OFFS_REG_S6 = const offset_of!(Context, regs.s6),
    CTX_OFFS_REG_S7 = const offset_of!(Context, regs.s7),
    CTX_OFFS_REG_S8 = const offset_of!(Context, regs.s8),
    CTX_OFFS_REG_S9 = const offset_of!(Context, regs.s9),
    CTX_OFFS_REG_S10 = const offset_of!(Context, regs.s10),
    CTX_OFFS_REG_S11 = const offset_of!(Context, regs.s11),
    CTX_OFFS_REG_T3 = const offset_of!(Context, regs.t3),
    CTX_OFFS_REG_T4 = const offset_of!(Context, regs.t4),
    CTX_OFFS_REG_T5 = const offset_of!(Context, regs.t5),
    CTX_OFFS_REG_T6 = const offset_of!(Context, regs.t6),

    CTX_OFFS_FREG_FT0 = const offset_of!(Context, fregs.ft0),
    CTX_OFFS_FREG_FT1 = const offset_of!(Context, fregs.ft1),
    CTX_OFFS_FREG_FT2 = const offset_of!(Context, fregs.ft2),
    CTX_OFFS_FREG_FT3 = const offset_of!(Context, fregs.ft3),
    CTX_OFFS_FREG_FT4 = const offset_of!(Context, fregs.ft4),
    CTX_OFFS_FREG_FT5 = const offset_of!(Context, fregs.ft5),
    CTX_OFFS_FREG_FT6 = const offset_of!(Context, fregs.ft6),
    CTX_OFFS_FREG_FT7 = const offset_of!(Context, fregs.ft7),
    CTX_OFFS_FREG_FS0 = const offset_of!(Context, fregs.fs0),
    CTX_OFFS_FREG_FS1 = const offset_of!(Context, fregs.fs1),
    CTX_OFFS_FREG_FA0 = const offset_of!(Context, fregs.fa0),
    CTX_OFFS_FREG_FA1 = const offset_of!(Context, fregs.fa1),
    CTX_OFFS_FREG_FA2 = const offset_of!(Context, fregs.fa2),
    CTX_OFFS_FREG_FA3 = const offset_of!(Context, fregs.fa3),
    CTX_OFFS_FREG_FA4 = const offset_of!(Context, fregs.fa4),
    CTX_OFFS_FREG_FA5 = const offset_of!(Context, fregs.fa5),
    CTX_OFFS_FREG_FA6 = const offset_of!(Context, fregs.fa6),
    CTX_OFFS_FREG_FA7 = const offset_of!(Context, fregs.fa7),
    CTX_OFFS_FREG_FS2 = const offset_of!(Context, fregs.fs2),
    CTX_OFFS_FREG_FS3 = const offset_of!(Context, fregs.fs3),
    CTX_OFFS_FREG_FS4 = const offset_of!(Context, fregs.fs4),
    CTX_OFFS_FREG_FS5 = const offset_of!(Context, fregs.fs5),
    CTX_OFFS_FREG_FS6 = const offset_of!(Context, fregs.fs6),
    CTX_OFFS_FREG_FS7 = const offset_of!(Context, fregs.fs7),
    CTX_OFFS_FREG_FS8 = const offset_of!(Context, fregs.fs8),
    CTX_OFFS_FREG_FS9 = const offset_of!(Context, fregs.fs9),
    CTX_OFFS_FREG_FS10 = const offset_of!(Context, fregs.fs10),
    CTX_OFFS_FREG_FS11 = const offset_of!(Context, fregs.fs11),
    CTX_OFFS_FREG_FT8 = const offset_of!(Context, fregs.ft8),
    CTX_OFFS_FREG_FT9 = const offset_of!(Context, fregs.ft9),
    CTX_OFFS_FREG_FT10 = const offset_of!(Context, fregs.ft10),
    CTX_OFFS_FREG_FT11 = const offset_of!(Context, fregs.ft11),

    CTX_OFFS_SSTATUS = const offset_of!(Context, sstatus),
    CTX_OFFS_SCAUSE = const offset_of!(Context, scause),
    CTX_OFFS_SIE = const offset_of!(Context, sie),
    CTX_OFFS_STVAL = const offset_of!(Context, stval),
    CTX_OFFS_SEPC = const offset_of!(Context, sepc),
}

#[cfg(target_arch = "riscv32")]
core::arch::global_asm! {
    r"
    .macro PUSH_REG reg, offset
        sw \reg, \offset(sp)
    .endm
    .macro POP_REG reg, offset
        lw \reg, \offset(sp)
    .endm
    .macro PUSH_FREG freg, offset
        fsw \freg, \offset(sp)
    .endm
    .macro POP_FREG freg, offset
        flw \freg, \offset(sp)
    .endm
    "
}

#[cfg(target_arch = "riscv64")]
core::arch::global_asm! {
    r"
    .macro PUSH_REG reg, offset
        sd \reg, \offset(sp)
    .endm
    .macro POP_REG reg, offset
        ld \reg, \offset(sp)
    .endm
    .macro PUSH_FREG freg, offset
        fsd \freg, \offset(sp)
    .endm
    .macro POP_FREG freg, offset
        fld \freg, \offset(sp)
    .endm
    "
}

core::arch::global_asm! {
    r"
    .align 12
    .global trap_entry
    trap_entry:
        csrrw sp, sscratch, sp
        bnez sp, trap_from_user_st

    trap_from_kern_st:
        csrr sp, sscratch
        addi sp, sp, -CTX_SIZE

    trap_from_user_st:
        PUSH_REG ra, CTX_OFFS_REG_RA
        PUSH_REG gp, CTX_OFFS_REG_GP
        PUSH_REG tp, CTX_OFFS_REG_TP
        PUSH_REG t0, CTX_OFFS_REG_T0
        PUSH_REG t1, CTX_OFFS_REG_T1
        PUSH_REG t2, CTX_OFFS_REG_T2
        PUSH_REG s0, CTX_OFFS_REG_S0
        PUSH_REG s1, CTX_OFFS_REG_S1
        PUSH_REG a0, CTX_OFFS_REG_A0
        PUSH_REG a1, CTX_OFFS_REG_A1
        PUSH_REG a2, CTX_OFFS_REG_A2
        PUSH_REG a3, CTX_OFFS_REG_A3
        PUSH_REG a4, CTX_OFFS_REG_A4
        PUSH_REG a5, CTX_OFFS_REG_A5
        PUSH_REG a6, CTX_OFFS_REG_A6
        PUSH_REG a7, CTX_OFFS_REG_A7
        PUSH_REG s2, CTX_OFFS_REG_S2
        PUSH_REG s3, CTX_OFFS_REG_S3
        PUSH_REG s4, CTX_OFFS_REG_S4
        PUSH_REG s5, CTX_OFFS_REG_S5
        PUSH_REG s6, CTX_OFFS_REG_S6
        PUSH_REG s7, CTX_OFFS_REG_S7
        PUSH_REG s8, CTX_OFFS_REG_S8
        PUSH_REG s9, CTX_OFFS_REG_S9
        PUSH_REG s10, CTX_OFFS_REG_S10
        PUSH_REG s11, CTX_OFFS_REG_S11
        PUSH_REG t3, CTX_OFFS_REG_T3
        PUSH_REG t4, CTX_OFFS_REG_T4
        PUSH_REG t5, CTX_OFFS_REG_T5
        PUSH_REG t6, CTX_OFFS_REG_T6

        PUSH_FREG ft0, CTX_OFFS_FREG_FT0
        PUSH_FREG ft1, CTX_OFFS_FREG_FT1
        PUSH_FREG ft2, CTX_OFFS_FREG_FT2
        PUSH_FREG ft3, CTX_OFFS_FREG_FT3
        PUSH_FREG ft4, CTX_OFFS_FREG_FT4
        PUSH_FREG ft5, CTX_OFFS_FREG_FT5
        PUSH_FREG ft6, CTX_OFFS_FREG_FT6
        PUSH_FREG ft7, CTX_OFFS_FREG_FT7
        PUSH_FREG fs0, CTX_OFFS_FREG_FS0
        PUSH_FREG fs1, CTX_OFFS_FREG_FS1
        PUSH_FREG fa0, CTX_OFFS_FREG_FA0
        PUSH_FREG fa1, CTX_OFFS_FREG_FA1
        PUSH_FREG fa2, CTX_OFFS_FREG_FA2
        PUSH_FREG fa3, CTX_OFFS_FREG_FA3
        PUSH_FREG fa4, CTX_OFFS_FREG_FA4
        PUSH_FREG fa5, CTX_OFFS_FREG_FA5
        PUSH_FREG fa6, CTX_OFFS_FREG_FA6
        PUSH_FREG fa7, CTX_OFFS_FREG_FA7
        PUSH_FREG fs2, CTX_OFFS_FREG_FS2
        PUSH_FREG fs3, CTX_OFFS_FREG_FS3
        PUSH_FREG fs4, CTX_OFFS_FREG_FS4
        PUSH_FREG fs5, CTX_OFFS_FREG_FS5
        PUSH_FREG fs6, CTX_OFFS_FREG_FS6
        PUSH_FREG fs7, CTX_OFFS_FREG_FS7
        PUSH_FREG fs8, CTX_OFFS_FREG_FS8
        PUSH_FREG fs9, CTX_OFFS_FREG_FS9
        PUSH_FREG fs10, CTX_OFFS_FREG_FS10
        PUSH_FREG fs11, CTX_OFFS_FREG_FS11
        PUSH_FREG ft8, CTX_OFFS_FREG_FT8
        PUSH_FREG ft9, CTX_OFFS_FREG_FT9
        PUSH_FREG ft10, CTX_OFFS_FREG_FT10
        PUSH_FREG ft11, CTX_OFFS_FREG_FT11

        csrrw t0, sscratch, zero
        PUSH_REG t0, CTX_OFFS_REG_SP

        csrr t1, sstatus
        PUSH_REG t1, CTX_OFFS_SSTATUS
        csrr t2, scause
        PUSH_REG t2, CTX_OFFS_SCAUSE
        csrr t3, sie
        PUSH_REG t3, CTX_OFFS_SIE
        csrr t4, stval
        PUSH_REG t4, CTX_OFFS_STVAL
        csrr t5, sepc
        PUSH_REG t5, CTX_OFFS_SEPC

        andi t1, t1, 1 << 8
        beqz t1, trap_from_user_ed

    trap_from_kern_ed:
        mv a0, sp
        call {KERNEL_TRAP_HANDLER}
        j trap_exit

    trap_from_user_ed:
        POP_REG sp, 0

        POP_REG s0, 0 * XLENB
        POP_REG s1, 1 * XLENB
        POP_REG s2, 2 * XLENB
        POP_REG s3, 3 * XLENB
        POP_REG s4, 4 * XLENB
        POP_REG s5, 5 * XLENB
        POP_REG s6, 6 * XLENB
        POP_REG s7, 7 * XLENB
        POP_REG s8, 8 * XLENB
        POP_REG s9, 9 * XLENB
        POP_REG s10, 10 * XLENB
        POP_REG s11, 11 * XLENB
        POP_REG ra, 12 * XLENB
        POP_REG gp, 13 * XLENB
        POP_REG tp, 14 * XLENB

        addi sp, sp, 15 * XLENB
        ret

    .global run_user
    run_user:
        addi sp, sp, -15 * XLENB

        PUSH_REG tp, 14 * XLENB
        PUSH_REG gp, 13 * XLENB
        PUSH_REG ra, 12 * XLENB
        PUSH_REG s11, 11 * XLENB
        PUSH_REG s10, 10 * XLENB
        PUSH_REG s9, 9 * XLENB
        PUSH_REG s8, 8 * XLENB
        PUSH_REG s7, 7 * XLENB
        PUSH_REG s6, 6 * XLENB
        PUSH_REG s5, 5 * XLENB
        PUSH_REG s4, 4 * XLENB
        PUSH_REG s3, 3 * XLENB
        PUSH_REG s2, 2 * XLENB
        PUSH_REG s1, 1 * XLENB
        PUSH_REG s0, 0 * XLENB

        mv t0, sp

        mv sp, a0
        PUSH_REG t0, 0

        csrw sscratch, sp

    trap_exit:
        POP_REG t5, CTX_OFFS_SEPC
        csrw sepc, t5
        POP_REG t4, CTX_OFFS_STVAL
        csrw stval, t4
        POP_REG t3, CTX_OFFS_SIE
        csrw sie, t3
        POP_REG t2, CTX_OFFS_SCAUSE
        csrw scause, t2
        POP_REG t1, CTX_OFFS_SSTATUS
        csrw sstatus, t1

        POP_FREG ft11, CTX_OFFS_FREG_FT11
        POP_FREG ft10, CTX_OFFS_FREG_FT10
        POP_FREG ft9, CTX_OFFS_FREG_FT9
        POP_FREG ft8, CTX_OFFS_FREG_FT8
        POP_FREG fs11, CTX_OFFS_FREG_FS11
        POP_FREG fs10, CTX_OFFS_FREG_FS10
        POP_FREG fs9, CTX_OFFS_FREG_FS9
        POP_FREG fs8, CTX_OFFS_FREG_FS8
        POP_FREG fs7, CTX_OFFS_FREG_FS7
        POP_FREG fs6, CTX_OFFS_FREG_FS6
        POP_FREG fs5, CTX_OFFS_FREG_FS5
        POP_FREG fs4, CTX_OFFS_FREG_FS4
        POP_FREG fs3, CTX_OFFS_FREG_FS3
        POP_FREG fs2, CTX_OFFS_FREG_FS2
        POP_FREG fa7, CTX_OFFS_FREG_FA7
        POP_FREG fa6, CTX_OFFS_FREG_FA6
        POP_FREG fa5, CTX_OFFS_FREG_FA5
        POP_FREG fa4, CTX_OFFS_FREG_FA4
        POP_FREG fa3, CTX_OFFS_FREG_FA3
        POP_FREG fa2, CTX_OFFS_FREG_FA2
        POP_FREG fa1, CTX_OFFS_FREG_FA1
        POP_FREG fa0, CTX_OFFS_FREG_FA0
        POP_FREG fs1, CTX_OFFS_FREG_FS1
        POP_FREG fs0, CTX_OFFS_FREG_FS0
        POP_FREG ft7, CTX_OFFS_FREG_FT7
        POP_FREG ft6, CTX_OFFS_FREG_FT6
        POP_FREG ft5, CTX_OFFS_FREG_FT5
        POP_FREG ft4, CTX_OFFS_FREG_FT4
        POP_FREG ft3, CTX_OFFS_FREG_FT3
        POP_FREG ft2, CTX_OFFS_FREG_FT2
        POP_FREG ft1, CTX_OFFS_FREG_FT1
        POP_FREG ft0, CTX_OFFS_FREG_FT0

        POP_REG t6, CTX_OFFS_REG_T6
        POP_REG t5, CTX_OFFS_REG_T5
        POP_REG t4, CTX_OFFS_REG_T4
        POP_REG t3, CTX_OFFS_REG_T3
        POP_REG s11, CTX_OFFS_REG_S11
        POP_REG s10, CTX_OFFS_REG_S10
        POP_REG s9, CTX_OFFS_REG_S9
        POP_REG s8, CTX_OFFS_REG_S8
        POP_REG s7, CTX_OFFS_REG_S7
        POP_REG s6, CTX_OFFS_REG_S6
        POP_REG s5, CTX_OFFS_REG_S5
        POP_REG s4, CTX_OFFS_REG_S4
        POP_REG s3, CTX_OFFS_REG_S3
        POP_REG s2, CTX_OFFS_REG_S2
        POP_REG a7, CTX_OFFS_REG_A7
        POP_REG a6, CTX_OFFS_REG_A6
        POP_REG a5, CTX_OFFS_REG_A5
        POP_REG a4, CTX_OFFS_REG_A4
        POP_REG a3, CTX_OFFS_REG_A3
        POP_REG a2, CTX_OFFS_REG_A2
        POP_REG a1, CTX_OFFS_REG_A1
        POP_REG a0, CTX_OFFS_REG_A0
        POP_REG s1, CTX_OFFS_REG_S1
        POP_REG s0, CTX_OFFS_REG_S0
        POP_REG t2, CTX_OFFS_REG_T2
        POP_REG t1, CTX_OFFS_REG_T1
        POP_REG t0, CTX_OFFS_REG_T0
        POP_REG tp, CTX_OFFS_REG_TP
        POP_REG gp, CTX_OFFS_REG_GP
        POP_REG ra, CTX_OFFS_REG_RA

        POP_REG sp, CTX_OFFS_REG_SP

        sret
    ",
    KERNEL_TRAP_HANDLER = sym handle_kern_trap,
}
