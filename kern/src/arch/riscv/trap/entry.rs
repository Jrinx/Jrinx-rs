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
    .macro ST_REG reg, offset
        sw \reg, \offset(sp)
    .endm
    .macro LD_REG reg, offset
        lw \reg, \offset(sp)
    .endm
    .macro ST_FREG freg, offset
        fsw \freg, \offset(sp)
    .endm
    .macro LD_FREG freg, offset
        flw \freg, \offset(sp)
    .endm
    "
}

#[cfg(target_arch = "riscv64")]
core::arch::global_asm! {
    r"
    .macro ST_REG reg, offset
        sd \reg, \offset(sp)
    .endm
    .macro LD_REG reg, offset
        ld \reg, \offset(sp)
    .endm
    .macro ST_FREG freg, offset
        fsd \freg, \offset(sp)
    .endm
    .macro LD_FREG freg, offset
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
        ST_REG ra, CTX_OFFS_REG_RA
        ST_REG gp, CTX_OFFS_REG_GP
        ST_REG tp, CTX_OFFS_REG_TP
        ST_REG t0, CTX_OFFS_REG_T0
        ST_REG t1, CTX_OFFS_REG_T1
        ST_REG t2, CTX_OFFS_REG_T2
        ST_REG s0, CTX_OFFS_REG_S0
        ST_REG s1, CTX_OFFS_REG_S1
        ST_REG a0, CTX_OFFS_REG_A0
        ST_REG a1, CTX_OFFS_REG_A1
        ST_REG a2, CTX_OFFS_REG_A2
        ST_REG a3, CTX_OFFS_REG_A3
        ST_REG a4, CTX_OFFS_REG_A4
        ST_REG a5, CTX_OFFS_REG_A5
        ST_REG a6, CTX_OFFS_REG_A6
        ST_REG a7, CTX_OFFS_REG_A7
        ST_REG s2, CTX_OFFS_REG_S2
        ST_REG s3, CTX_OFFS_REG_S3
        ST_REG s4, CTX_OFFS_REG_S4
        ST_REG s5, CTX_OFFS_REG_S5
        ST_REG s6, CTX_OFFS_REG_S6
        ST_REG s7, CTX_OFFS_REG_S7
        ST_REG s8, CTX_OFFS_REG_S8
        ST_REG s9, CTX_OFFS_REG_S9
        ST_REG s10, CTX_OFFS_REG_S10
        ST_REG s11, CTX_OFFS_REG_S11
        ST_REG t3, CTX_OFFS_REG_T3
        ST_REG t4, CTX_OFFS_REG_T4
        ST_REG t5, CTX_OFFS_REG_T5
        ST_REG t6, CTX_OFFS_REG_T6

        ST_FREG ft0, CTX_OFFS_FREG_FT0
        ST_FREG ft1, CTX_OFFS_FREG_FT1
        ST_FREG ft2, CTX_OFFS_FREG_FT2
        ST_FREG ft3, CTX_OFFS_FREG_FT3
        ST_FREG ft4, CTX_OFFS_FREG_FT4
        ST_FREG ft5, CTX_OFFS_FREG_FT5
        ST_FREG ft6, CTX_OFFS_FREG_FT6
        ST_FREG ft7, CTX_OFFS_FREG_FT7
        ST_FREG fs0, CTX_OFFS_FREG_FS0
        ST_FREG fs1, CTX_OFFS_FREG_FS1
        ST_FREG fa0, CTX_OFFS_FREG_FA0
        ST_FREG fa1, CTX_OFFS_FREG_FA1
        ST_FREG fa2, CTX_OFFS_FREG_FA2
        ST_FREG fa3, CTX_OFFS_FREG_FA3
        ST_FREG fa4, CTX_OFFS_FREG_FA4
        ST_FREG fa5, CTX_OFFS_FREG_FA5
        ST_FREG fa6, CTX_OFFS_FREG_FA6
        ST_FREG fa7, CTX_OFFS_FREG_FA7
        ST_FREG fs2, CTX_OFFS_FREG_FS2
        ST_FREG fs3, CTX_OFFS_FREG_FS3
        ST_FREG fs4, CTX_OFFS_FREG_FS4
        ST_FREG fs5, CTX_OFFS_FREG_FS5
        ST_FREG fs6, CTX_OFFS_FREG_FS6
        ST_FREG fs7, CTX_OFFS_FREG_FS7
        ST_FREG fs8, CTX_OFFS_FREG_FS8
        ST_FREG fs9, CTX_OFFS_FREG_FS9
        ST_FREG fs10, CTX_OFFS_FREG_FS10
        ST_FREG fs11, CTX_OFFS_FREG_FS11
        ST_FREG ft8, CTX_OFFS_FREG_FT8
        ST_FREG ft9, CTX_OFFS_FREG_FT9
        ST_FREG ft10, CTX_OFFS_FREG_FT10
        ST_FREG ft11, CTX_OFFS_FREG_FT11

        csrrw t0, sscratch, zero
        ST_REG t0, CTX_OFFS_REG_SP

        csrr t1, sstatus
        ST_REG t1, CTX_OFFS_SSTATUS
        csrr t2, scause
        ST_REG t2, CTX_OFFS_SCAUSE
        csrr t3, sie
        ST_REG t3, CTX_OFFS_SIE
        csrr t4, stval
        ST_REG t4, CTX_OFFS_STVAL
        csrr t5, sepc
        ST_REG t5, CTX_OFFS_SEPC

        andi t1, t1, 1 << 8
        beqz t1, trap_from_user_ed

    trap_from_kern_ed:
        mv a0, sp
        call {KERNEL_TRAP_HANDLER}
        j trap_exit

    trap_from_user_ed:
        LD_REG sp, 0

        LD_REG s0, 0 * XLENB
        LD_REG s1, 1 * XLENB
        LD_REG s2, 2 * XLENB
        LD_REG s3, 3 * XLENB
        LD_REG s4, 4 * XLENB
        LD_REG s5, 5 * XLENB
        LD_REG s6, 6 * XLENB
        LD_REG s7, 7 * XLENB
        LD_REG s8, 8 * XLENB
        LD_REG s9, 9 * XLENB
        LD_REG s10, 10 * XLENB
        LD_REG s11, 11 * XLENB
        LD_REG ra, 12 * XLENB
        LD_REG tp, 13 * XLENB

        addi sp, sp, 14 * XLENB
        ret

    .global run_user
    run_user:
        addi sp, sp, -14 * XLENB

        ST_REG tp, 13 * XLENB
        ST_REG ra, 12 * XLENB
        ST_REG s11, 11 * XLENB
        ST_REG s10, 10 * XLENB
        ST_REG s9, 9 * XLENB
        ST_REG s8, 8 * XLENB
        ST_REG s7, 7 * XLENB
        ST_REG s6, 6 * XLENB
        ST_REG s5, 5 * XLENB
        ST_REG s4, 4 * XLENB
        ST_REG s3, 3 * XLENB
        ST_REG s2, 2 * XLENB
        ST_REG s1, 1 * XLENB
        ST_REG s0, 0 * XLENB

        ST_REG sp, 0

        mv sp, a0
        csrw sscratch, sp

    trap_exit:
        LD_REG t5, CTX_OFFS_SEPC
        csrw sepc, t5
        LD_REG t4, CTX_OFFS_STVAL
        csrw stval, t4
        LD_REG t3, CTX_OFFS_SIE
        csrw sie, t3
        LD_REG t2, CTX_OFFS_SCAUSE
        csrw scause, t2
        LD_REG t1, CTX_OFFS_SSTATUS
        csrw sstatus, t1

        LD_FREG ft11, CTX_OFFS_FREG_FT11
        LD_FREG ft10, CTX_OFFS_FREG_FT10
        LD_FREG ft9, CTX_OFFS_FREG_FT9
        LD_FREG ft8, CTX_OFFS_FREG_FT8
        LD_FREG fs11, CTX_OFFS_FREG_FS11
        LD_FREG fs10, CTX_OFFS_FREG_FS10
        LD_FREG fs9, CTX_OFFS_FREG_FS9
        LD_FREG fs8, CTX_OFFS_FREG_FS8
        LD_FREG fs7, CTX_OFFS_FREG_FS7
        LD_FREG fs6, CTX_OFFS_FREG_FS6
        LD_FREG fs5, CTX_OFFS_FREG_FS5
        LD_FREG fs4, CTX_OFFS_FREG_FS4
        LD_FREG fs3, CTX_OFFS_FREG_FS3
        LD_FREG fs2, CTX_OFFS_FREG_FS2
        LD_FREG fa7, CTX_OFFS_FREG_FA7
        LD_FREG fa6, CTX_OFFS_FREG_FA6
        LD_FREG fa5, CTX_OFFS_FREG_FA5
        LD_FREG fa4, CTX_OFFS_FREG_FA4
        LD_FREG fa3, CTX_OFFS_FREG_FA3
        LD_FREG fa2, CTX_OFFS_FREG_FA2
        LD_FREG fa1, CTX_OFFS_FREG_FA1
        LD_FREG fa0, CTX_OFFS_FREG_FA0
        LD_FREG fs1, CTX_OFFS_FREG_FS1
        LD_FREG fs0, CTX_OFFS_FREG_FS0
        LD_FREG ft7, CTX_OFFS_FREG_FT7
        LD_FREG ft6, CTX_OFFS_FREG_FT6
        LD_FREG ft5, CTX_OFFS_FREG_FT5
        LD_FREG ft4, CTX_OFFS_FREG_FT4
        LD_FREG ft3, CTX_OFFS_FREG_FT3
        LD_FREG ft2, CTX_OFFS_FREG_FT2
        LD_FREG ft1, CTX_OFFS_FREG_FT1
        LD_FREG ft0, CTX_OFFS_FREG_FT0

        LD_REG t6, CTX_OFFS_REG_T6
        LD_REG t5, CTX_OFFS_REG_T5
        LD_REG t4, CTX_OFFS_REG_T4
        LD_REG t3, CTX_OFFS_REG_T3
        LD_REG s11, CTX_OFFS_REG_S11
        LD_REG s10, CTX_OFFS_REG_S10
        LD_REG s9, CTX_OFFS_REG_S9
        LD_REG s8, CTX_OFFS_REG_S8
        LD_REG s7, CTX_OFFS_REG_S7
        LD_REG s6, CTX_OFFS_REG_S6
        LD_REG s5, CTX_OFFS_REG_S5
        LD_REG s4, CTX_OFFS_REG_S4
        LD_REG s3, CTX_OFFS_REG_S3
        LD_REG s2, CTX_OFFS_REG_S2
        LD_REG a7, CTX_OFFS_REG_A7
        LD_REG a6, CTX_OFFS_REG_A6
        LD_REG a5, CTX_OFFS_REG_A5
        LD_REG a4, CTX_OFFS_REG_A4
        LD_REG a3, CTX_OFFS_REG_A3
        LD_REG a2, CTX_OFFS_REG_A2
        LD_REG a1, CTX_OFFS_REG_A1
        LD_REG a0, CTX_OFFS_REG_A0
        LD_REG s1, CTX_OFFS_REG_S1
        LD_REG s0, CTX_OFFS_REG_S0
        LD_REG t2, CTX_OFFS_REG_T2
        LD_REG t1, CTX_OFFS_REG_T1
        LD_REG t0, CTX_OFFS_REG_T0
        LD_REG tp, CTX_OFFS_REG_TP
        LD_REG gp, CTX_OFFS_REG_GP
        LD_REG ra, CTX_OFFS_REG_RA

        LD_REG sp, CTX_OFFS_REG_SP

        sret
    ",
    KERNEL_TRAP_HANDLER = sym handle_kern_trap,
}
