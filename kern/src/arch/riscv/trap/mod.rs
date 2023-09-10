mod trap;
pub use trap::*;

use core::mem::{offset_of, size_of};

use riscv::register::{
    scause::{Exception, Trap},
    sie::Sie,
    sstatus::Sstatus,
    utvec::TrapMode,
};

#[derive(Debug, Default, Clone, Copy)]
#[repr(C)]
pub struct Register {
    zero: usize,
    ra: usize,
    sp: usize,
    gp: usize,
    tp: usize,
    t0: usize,
    t1: usize,
    t2: usize,
    s0: usize,
    s1: usize,
    a0: usize,
    a1: usize,
    a2: usize,
    a3: usize,
    a4: usize,
    a5: usize,
    a6: usize,
    a7: usize,
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
    t3: usize,
    t4: usize,
    t5: usize,
    t6: usize,
}

#[derive(Debug, Default, Clone, Copy)]
#[repr(C)]
pub struct FRegister {
    ft0: usize,
    ft1: usize,
    ft2: usize,
    ft3: usize,
    ft4: usize,
    ft5: usize,
    ft6: usize,
    ft7: usize,
    fs0: usize,
    fs1: usize,
    fa0: usize,
    fa1: usize,
    fa2: usize,
    fa3: usize,
    fa4: usize,
    fa5: usize,
    fa6: usize,
    fa7: usize,
    fs2: usize,
    fs3: usize,
    fs4: usize,
    fs5: usize,
    fs6: usize,
    fs7: usize,
    fs8: usize,
    fs9: usize,
    fs10: usize,
    fs11: usize,
    ft8: usize,
    ft9: usize,
    ft10: usize,
    ft11: usize,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct Trapframe {
    regs: Register,
    fregs: FRegister,
    sstatus: Sstatus,
    sie: Sie,
    sepc: usize,
}

pub(in crate::arch) fn init() {
    extern "C" {
        fn trap_entry();
    }
    unsafe {
        riscv::register::sscratch::write(0);
        riscv::register::stvec::write(trap_entry as usize, TrapMode::Direct);
    }
}

fn handle_kern_trap(tf: &mut Trapframe) {
    let cause = riscv::register::scause::read().cause();
    trace!("kernel trap: {:?}", cause);
    match cause {
        Trap::Exception(Exception::Breakpoint) => trap::handle_breakpoint(&mut tf.sepc),
        _ => todo!(),
    }
}

core::arch::global_asm! {
    r#".attribute arch, "rv64imafd""#,
    r"
    .equ XLENB, {XLENB}
    .equ TF_SIZE, {TF_SIZE}
    .equ TF_OFFS_REG_ZERO, {TF_OFFS_REG_ZERO}
    .equ TF_OFFS_REG_RA, {TF_OFFS_REG_RA}
    .equ TF_OFFS_REG_SP, {TF_OFFS_REG_SP}
    .equ TF_OFFS_REG_GP, {TF_OFFS_REG_GP}
    .equ TF_OFFS_REG_TP, {TF_OFFS_REG_TP}
    .equ TF_OFFS_REG_T0, {TF_OFFS_REG_T0}
    .equ TF_OFFS_REG_T1, {TF_OFFS_REG_T1}
    .equ TF_OFFS_REG_T2, {TF_OFFS_REG_T2}
    .equ TF_OFFS_REG_S0, {TF_OFFS_REG_S0}
    .equ TF_OFFS_REG_S1, {TF_OFFS_REG_S1}
    .equ TF_OFFS_REG_A0, {TF_OFFS_REG_A0}
    .equ TF_OFFS_REG_A1, {TF_OFFS_REG_A1}
    .equ TF_OFFS_REG_A2, {TF_OFFS_REG_A2}
    .equ TF_OFFS_REG_A3, {TF_OFFS_REG_A3}
    .equ TF_OFFS_REG_A4, {TF_OFFS_REG_A4}
    .equ TF_OFFS_REG_A5, {TF_OFFS_REG_A5}
    .equ TF_OFFS_REG_A6, {TF_OFFS_REG_A6}
    .equ TF_OFFS_REG_A7, {TF_OFFS_REG_A7}
    .equ TF_OFFS_REG_S2, {TF_OFFS_REG_S2}
    .equ TF_OFFS_REG_S3, {TF_OFFS_REG_S3}
    .equ TF_OFFS_REG_S4, {TF_OFFS_REG_S4}
    .equ TF_OFFS_REG_S5, {TF_OFFS_REG_S5}
    .equ TF_OFFS_REG_S6, {TF_OFFS_REG_S6}
    .equ TF_OFFS_REG_S7, {TF_OFFS_REG_S7}
    .equ TF_OFFS_REG_S8, {TF_OFFS_REG_S8}
    .equ TF_OFFS_REG_S9, {TF_OFFS_REG_S9}
    .equ TF_OFFS_REG_S10, {TF_OFFS_REG_S10}
    .equ TF_OFFS_REG_S11, {TF_OFFS_REG_S11}
    .equ TF_OFFS_REG_T3, {TF_OFFS_REG_T3}
    .equ TF_OFFS_REG_T4, {TF_OFFS_REG_T4}
    .equ TF_OFFS_REG_T5, {TF_OFFS_REG_T5}
    .equ TF_OFFS_REG_T6, {TF_OFFS_REG_T6}

    .equ TF_OFFS_FREG_FT0, {TF_OFFS_FREG_FT0}
    .equ TF_OFFS_FREG_FT1, {TF_OFFS_FREG_FT1}
    .equ TF_OFFS_FREG_FT2, {TF_OFFS_FREG_FT2}
    .equ TF_OFFS_FREG_FT3, {TF_OFFS_FREG_FT3}
    .equ TF_OFFS_FREG_FT4, {TF_OFFS_FREG_FT4}
    .equ TF_OFFS_FREG_FT5, {TF_OFFS_FREG_FT5}
    .equ TF_OFFS_FREG_FT6, {TF_OFFS_FREG_FT6}
    .equ TF_OFFS_FREG_FT7, {TF_OFFS_FREG_FT7}
    .equ TF_OFFS_FREG_FS0, {TF_OFFS_FREG_FS0}
    .equ TF_OFFS_FREG_FS1, {TF_OFFS_FREG_FS1}
    .equ TF_OFFS_FREG_FA0, {TF_OFFS_FREG_FA0}
    .equ TF_OFFS_FREG_FA1, {TF_OFFS_FREG_FA1}
    .equ TF_OFFS_FREG_FA2, {TF_OFFS_FREG_FA2}
    .equ TF_OFFS_FREG_FA3, {TF_OFFS_FREG_FA3}
    .equ TF_OFFS_FREG_FA4, {TF_OFFS_FREG_FA4}
    .equ TF_OFFS_FREG_FA5, {TF_OFFS_FREG_FA5}
    .equ TF_OFFS_FREG_FA6, {TF_OFFS_FREG_FA6}
    .equ TF_OFFS_FREG_FA7, {TF_OFFS_FREG_FA7}
    .equ TF_OFFS_FREG_FS2, {TF_OFFS_FREG_FS2}
    .equ TF_OFFS_FREG_FS3, {TF_OFFS_FREG_FS3}
    .equ TF_OFFS_FREG_FS4, {TF_OFFS_FREG_FS4}
    .equ TF_OFFS_FREG_FS5, {TF_OFFS_FREG_FS5}
    .equ TF_OFFS_FREG_FS6, {TF_OFFS_FREG_FS6}
    .equ TF_OFFS_FREG_FS7, {TF_OFFS_FREG_FS7}
    .equ TF_OFFS_FREG_FS8, {TF_OFFS_FREG_FS8}
    .equ TF_OFFS_FREG_FS9, {TF_OFFS_FREG_FS9}
    .equ TF_OFFS_FREG_FS10, {TF_OFFS_FREG_FS10}
    .equ TF_OFFS_FREG_FS11, {TF_OFFS_FREG_FS11}
    .equ TF_OFFS_FREG_FT8, {TF_OFFS_FREG_FT8}
    .equ TF_OFFS_FREG_FT9, {TF_OFFS_FREG_FT9}
    .equ TF_OFFS_FREG_FT10, {TF_OFFS_FREG_FT10}
    .equ TF_OFFS_FREG_FT11, {TF_OFFS_FREG_FT11}

    .equ TF_OFFS_SSTATUS, {TF_OFFS_SSTATUS}
    .equ TF_OFFS_SIE, {TF_OFFS_SIE}
    .equ TF_OFFS_SEPC, {TF_OFFS_SEPC}
    ",
    XLENB = const size_of::<usize>(),
    TF_SIZE = const size_of::<Trapframe>(),
    TF_OFFS_REG_ZERO = const offset_of!(Trapframe, regs.zero),
    TF_OFFS_REG_RA = const offset_of!(Trapframe, regs.ra),
    TF_OFFS_REG_SP = const offset_of!(Trapframe, regs.sp),
    TF_OFFS_REG_GP = const offset_of!(Trapframe, regs.gp),
    TF_OFFS_REG_TP = const offset_of!(Trapframe, regs.tp),
    TF_OFFS_REG_T0 = const offset_of!(Trapframe, regs.t0),
    TF_OFFS_REG_T1 = const offset_of!(Trapframe, regs.t1),
    TF_OFFS_REG_T2 = const offset_of!(Trapframe, regs.t2),
    TF_OFFS_REG_S0 = const offset_of!(Trapframe, regs.s0),
    TF_OFFS_REG_S1 = const offset_of!(Trapframe, regs.s1),
    TF_OFFS_REG_A0 = const offset_of!(Trapframe, regs.a0),
    TF_OFFS_REG_A1 = const offset_of!(Trapframe, regs.a1),
    TF_OFFS_REG_A2 = const offset_of!(Trapframe, regs.a2),
    TF_OFFS_REG_A3 = const offset_of!(Trapframe, regs.a3),
    TF_OFFS_REG_A4 = const offset_of!(Trapframe, regs.a4),
    TF_OFFS_REG_A5 = const offset_of!(Trapframe, regs.a5),
    TF_OFFS_REG_A6 = const offset_of!(Trapframe, regs.a6),
    TF_OFFS_REG_A7 = const offset_of!(Trapframe, regs.a7),
    TF_OFFS_REG_S2 = const offset_of!(Trapframe, regs.s2),
    TF_OFFS_REG_S3 = const offset_of!(Trapframe, regs.s3),
    TF_OFFS_REG_S4 = const offset_of!(Trapframe, regs.s4),
    TF_OFFS_REG_S5 = const offset_of!(Trapframe, regs.s5),
    TF_OFFS_REG_S6 = const offset_of!(Trapframe, regs.s6),
    TF_OFFS_REG_S7 = const offset_of!(Trapframe, regs.s7),
    TF_OFFS_REG_S8 = const offset_of!(Trapframe, regs.s8),
    TF_OFFS_REG_S9 = const offset_of!(Trapframe, regs.s9),
    TF_OFFS_REG_S10 = const offset_of!(Trapframe, regs.s10),
    TF_OFFS_REG_S11 = const offset_of!(Trapframe, regs.s11),
    TF_OFFS_REG_T3 = const offset_of!(Trapframe, regs.t3),
    TF_OFFS_REG_T4 = const offset_of!(Trapframe, regs.t4),
    TF_OFFS_REG_T5 = const offset_of!(Trapframe, regs.t5),
    TF_OFFS_REG_T6 = const offset_of!(Trapframe, regs.t6),

    TF_OFFS_FREG_FT0 = const offset_of!(Trapframe, fregs.ft0),
    TF_OFFS_FREG_FT1 = const offset_of!(Trapframe, fregs.ft1),
    TF_OFFS_FREG_FT2 = const offset_of!(Trapframe, fregs.ft2),
    TF_OFFS_FREG_FT3 = const offset_of!(Trapframe, fregs.ft3),
    TF_OFFS_FREG_FT4 = const offset_of!(Trapframe, fregs.ft4),
    TF_OFFS_FREG_FT5 = const offset_of!(Trapframe, fregs.ft5),
    TF_OFFS_FREG_FT6 = const offset_of!(Trapframe, fregs.ft6),
    TF_OFFS_FREG_FT7 = const offset_of!(Trapframe, fregs.ft7),
    TF_OFFS_FREG_FS0 = const offset_of!(Trapframe, fregs.fs0),
    TF_OFFS_FREG_FS1 = const offset_of!(Trapframe, fregs.fs1),
    TF_OFFS_FREG_FA0 = const offset_of!(Trapframe, fregs.fa0),
    TF_OFFS_FREG_FA1 = const offset_of!(Trapframe, fregs.fa1),
    TF_OFFS_FREG_FA2 = const offset_of!(Trapframe, fregs.fa2),
    TF_OFFS_FREG_FA3 = const offset_of!(Trapframe, fregs.fa3),
    TF_OFFS_FREG_FA4 = const offset_of!(Trapframe, fregs.fa4),
    TF_OFFS_FREG_FA5 = const offset_of!(Trapframe, fregs.fa5),
    TF_OFFS_FREG_FA6 = const offset_of!(Trapframe, fregs.fa6),
    TF_OFFS_FREG_FA7 = const offset_of!(Trapframe, fregs.fa7),
    TF_OFFS_FREG_FS2 = const offset_of!(Trapframe, fregs.fs2),
    TF_OFFS_FREG_FS3 = const offset_of!(Trapframe, fregs.fs3),
    TF_OFFS_FREG_FS4 = const offset_of!(Trapframe, fregs.fs4),
    TF_OFFS_FREG_FS5 = const offset_of!(Trapframe, fregs.fs5),
    TF_OFFS_FREG_FS6 = const offset_of!(Trapframe, fregs.fs6),
    TF_OFFS_FREG_FS7 = const offset_of!(Trapframe, fregs.fs7),
    TF_OFFS_FREG_FS8 = const offset_of!(Trapframe, fregs.fs8),
    TF_OFFS_FREG_FS9 = const offset_of!(Trapframe, fregs.fs9),
    TF_OFFS_FREG_FS10 = const offset_of!(Trapframe, fregs.fs10),
    TF_OFFS_FREG_FS11 = const offset_of!(Trapframe, fregs.fs11),
    TF_OFFS_FREG_FT8 = const offset_of!(Trapframe, fregs.ft8),
    TF_OFFS_FREG_FT9 = const offset_of!(Trapframe, fregs.ft9),
    TF_OFFS_FREG_FT10 = const offset_of!(Trapframe, fregs.ft10),
    TF_OFFS_FREG_FT11 = const offset_of!(Trapframe, fregs.ft11),

    TF_OFFS_SSTATUS = const offset_of!(Trapframe, sstatus),
    TF_OFFS_SIE = const offset_of!(Trapframe, sie),
    TF_OFFS_SEPC = const offset_of!(Trapframe, sepc),
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
        addi sp, sp, -TF_SIZE

    trap_from_user_st:
        ST_REG ra, TF_OFFS_REG_RA
        ST_REG gp, TF_OFFS_REG_GP
        ST_REG tp, TF_OFFS_REG_TP
        ST_REG t0, TF_OFFS_REG_T0
        ST_REG t1, TF_OFFS_REG_T1
        ST_REG t2, TF_OFFS_REG_T2
        ST_REG s0, TF_OFFS_REG_S0
        ST_REG s1, TF_OFFS_REG_S1
        ST_REG a0, TF_OFFS_REG_A0
        ST_REG a1, TF_OFFS_REG_A1
        ST_REG a2, TF_OFFS_REG_A2
        ST_REG a3, TF_OFFS_REG_A3
        ST_REG a4, TF_OFFS_REG_A4
        ST_REG a5, TF_OFFS_REG_A5
        ST_REG a6, TF_OFFS_REG_A6
        ST_REG a7, TF_OFFS_REG_A7
        ST_REG s2, TF_OFFS_REG_S2
        ST_REG s3, TF_OFFS_REG_S3
        ST_REG s4, TF_OFFS_REG_S4
        ST_REG s5, TF_OFFS_REG_S5
        ST_REG s6, TF_OFFS_REG_S6
        ST_REG s7, TF_OFFS_REG_S7
        ST_REG s8, TF_OFFS_REG_S8
        ST_REG s9, TF_OFFS_REG_S9
        ST_REG s10, TF_OFFS_REG_S10
        ST_REG s11, TF_OFFS_REG_S11
        ST_REG t3, TF_OFFS_REG_T3
        ST_REG t4, TF_OFFS_REG_T4
        ST_REG t5, TF_OFFS_REG_T5
        ST_REG t6, TF_OFFS_REG_T6

        ST_FREG ft0, TF_OFFS_FREG_FT0
        ST_FREG ft1, TF_OFFS_FREG_FT1
        ST_FREG ft2, TF_OFFS_FREG_FT2
        ST_FREG ft3, TF_OFFS_FREG_FT3
        ST_FREG ft4, TF_OFFS_FREG_FT4
        ST_FREG ft5, TF_OFFS_FREG_FT5
        ST_FREG ft6, TF_OFFS_FREG_FT6
        ST_FREG ft7, TF_OFFS_FREG_FT7
        ST_FREG fs0, TF_OFFS_FREG_FS0
        ST_FREG fs1, TF_OFFS_FREG_FS1
        ST_FREG fa0, TF_OFFS_FREG_FA0
        ST_FREG fa1, TF_OFFS_FREG_FA1
        ST_FREG fa2, TF_OFFS_FREG_FA2
        ST_FREG fa3, TF_OFFS_FREG_FA3
        ST_FREG fa4, TF_OFFS_FREG_FA4
        ST_FREG fa5, TF_OFFS_FREG_FA5
        ST_FREG fa6, TF_OFFS_FREG_FA6
        ST_FREG fa7, TF_OFFS_FREG_FA7
        ST_FREG fs2, TF_OFFS_FREG_FS2
        ST_FREG fs3, TF_OFFS_FREG_FS3
        ST_FREG fs4, TF_OFFS_FREG_FS4
        ST_FREG fs5, TF_OFFS_FREG_FS5
        ST_FREG fs6, TF_OFFS_FREG_FS6
        ST_FREG fs7, TF_OFFS_FREG_FS7
        ST_FREG fs8, TF_OFFS_FREG_FS8
        ST_FREG fs9, TF_OFFS_FREG_FS9
        ST_FREG fs10, TF_OFFS_FREG_FS10
        ST_FREG fs11, TF_OFFS_FREG_FS11
        ST_FREG ft8, TF_OFFS_FREG_FT8
        ST_FREG ft9, TF_OFFS_FREG_FT9
        ST_FREG ft10, TF_OFFS_FREG_FT10
        ST_FREG ft11, TF_OFFS_FREG_FT11

        csrrw t0, sscratch, zero
        ST_REG t0, TF_OFFS_REG_SP

        csrr t1, sstatus
        ST_REG t1, TF_OFFS_SSTATUS
        csrr t2, sie
        ST_REG t2, TF_OFFS_SIE
        csrr t3, sepc
        ST_REG t3, TF_OFFS_SEPC

        andi t1, t1, 1 << 8
        beqz t1, trap_from_user_ed

    trap_from_kern_ed:
        mv a0, sp
        call {TRAP_HANDLER}
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
        LD_REG t3, TF_OFFS_SEPC
        csrw sepc, t3
        LD_REG t2, TF_OFFS_SIE
        csrw sie, t2
        LD_REG t1, TF_OFFS_SSTATUS
        csrw sstatus, t1

        LD_FREG ft11, TF_OFFS_FREG_FT11
        LD_FREG ft10, TF_OFFS_FREG_FT10
        LD_FREG ft9, TF_OFFS_FREG_FT9
        LD_FREG ft8, TF_OFFS_FREG_FT8
        LD_FREG fs11, TF_OFFS_FREG_FS11
        LD_FREG fs10, TF_OFFS_FREG_FS10
        LD_FREG fs9, TF_OFFS_FREG_FS9
        LD_FREG fs8, TF_OFFS_FREG_FS8
        LD_FREG fs7, TF_OFFS_FREG_FS7
        LD_FREG fs6, TF_OFFS_FREG_FS6
        LD_FREG fs5, TF_OFFS_FREG_FS5
        LD_FREG fs4, TF_OFFS_FREG_FS4
        LD_FREG fs3, TF_OFFS_FREG_FS3
        LD_FREG fs2, TF_OFFS_FREG_FS2
        LD_FREG fa7, TF_OFFS_FREG_FA7
        LD_FREG fa6, TF_OFFS_FREG_FA6
        LD_FREG fa5, TF_OFFS_FREG_FA5
        LD_FREG fa4, TF_OFFS_FREG_FA4
        LD_FREG fa3, TF_OFFS_FREG_FA3
        LD_FREG fa2, TF_OFFS_FREG_FA2
        LD_FREG fa1, TF_OFFS_FREG_FA1
        LD_FREG fa0, TF_OFFS_FREG_FA0
        LD_FREG fs1, TF_OFFS_FREG_FS1
        LD_FREG fs0, TF_OFFS_FREG_FS0
        LD_FREG ft7, TF_OFFS_FREG_FT7
        LD_FREG ft6, TF_OFFS_FREG_FT6
        LD_FREG ft5, TF_OFFS_FREG_FT5
        LD_FREG ft4, TF_OFFS_FREG_FT4
        LD_FREG ft3, TF_OFFS_FREG_FT3
        LD_FREG ft2, TF_OFFS_FREG_FT2
        LD_FREG ft1, TF_OFFS_FREG_FT1
        LD_FREG ft0, TF_OFFS_FREG_FT0

        LD_REG t6, TF_OFFS_REG_T6
        LD_REG t5, TF_OFFS_REG_T5
        LD_REG t4, TF_OFFS_REG_T4
        LD_REG t3, TF_OFFS_REG_T3
        LD_REG s11, TF_OFFS_REG_S11
        LD_REG s10, TF_OFFS_REG_S10
        LD_REG s9, TF_OFFS_REG_S9
        LD_REG s8, TF_OFFS_REG_S8
        LD_REG s7, TF_OFFS_REG_S7
        LD_REG s6, TF_OFFS_REG_S6
        LD_REG s5, TF_OFFS_REG_S5
        LD_REG s4, TF_OFFS_REG_S4
        LD_REG s3, TF_OFFS_REG_S3
        LD_REG s2, TF_OFFS_REG_S2
        LD_REG a7, TF_OFFS_REG_A7
        LD_REG a6, TF_OFFS_REG_A6
        LD_REG a5, TF_OFFS_REG_A5
        LD_REG a4, TF_OFFS_REG_A4
        LD_REG a3, TF_OFFS_REG_A3
        LD_REG a2, TF_OFFS_REG_A2
        LD_REG a1, TF_OFFS_REG_A1
        LD_REG a0, TF_OFFS_REG_A0
        LD_REG s1, TF_OFFS_REG_S1
        LD_REG s0, TF_OFFS_REG_S0
        LD_REG t2, TF_OFFS_REG_T2
        LD_REG t1, TF_OFFS_REG_T1
        LD_REG t0, TF_OFFS_REG_T0
        LD_REG tp, TF_OFFS_REG_TP
        LD_REG gp, TF_OFFS_REG_GP
        LD_REG ra, TF_OFFS_REG_RA

        LD_REG sp, TF_OFFS_REG_SP

        sret
    ",
    TRAP_HANDLER = sym handle_kern_trap,
}
