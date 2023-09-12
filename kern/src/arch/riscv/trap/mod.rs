mod entry;

use core::mem::size_of;

use riscv::register::{
    scause::{Exception, Interrupt},
    utvec::TrapMode,
};

use crate::{
    arch::AbstractContext,
    mm::virt::VirtAddr,
    trap::{breakpoint, TrapReason},
};

use super::mm::virt::PagePerm;

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

#[derive(Debug, Default, Clone, Copy)]
#[repr(C)]
pub struct Context {
    regs: Register,
    fregs: FRegister,
    sstatus: usize,
    scause: usize,
    sie: usize,
    stval: usize,
    sepc: usize,
}

impl AbstractContext for Context {
    fn run(&mut self) {
        extern "C" {
            fn run_user(ctx: &mut Context);
        }
        unsafe {
            run_user(self);
        }
    }

    fn trap_reason(&self) -> TrapReason {
        let cause = self.scause;
        let is_interrupt = (cause & (1 << (size_of::<usize>() * 8 - 1))) != 0;
        if is_interrupt {
            let code = Interrupt::from(cause & !(1 << (size_of::<usize>() * 8 - 1)));
            TrapReason::Interrupt(code as usize)
        } else {
            let code = Exception::from(cause);
            match code {
                Exception::Breakpoint => TrapReason::Breakpoint {
                    addr: VirtAddr::new(self.sepc),
                },
                Exception::LoadPageFault => TrapReason::PageFault {
                    addr: VirtAddr::new(self.stval),
                    perm: PagePerm::R,
                },
                Exception::StorePageFault => TrapReason::PageFault {
                    addr: VirtAddr::new(self.stval),
                    perm: PagePerm::W,
                },
                Exception::InstructionPageFault => TrapReason::PageFault {
                    addr: VirtAddr::new(self.stval),
                    perm: PagePerm::X,
                },
                _ => TrapReason::Unknown,
            }
        }
    }

    fn acc_pc(&mut self) {
        let is_rvc = (unsafe { core::ptr::read_volatile(self.sepc as *const u8) & 0b11 }) != 0b11;
        if is_rvc {
            self.sepc += 2;
        } else {
            self.sepc += 4;
        }
    }
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

fn handle_kern_trap(ctx: &mut Context) {
    let reason = ctx.trap_reason();
    trace!("kernel trap: {:?}", reason);
    match reason {
        TrapReason::Breakpoint { addr: _ } => breakpoint::handle(ctx),
        _ => todo!(),
    }
}
