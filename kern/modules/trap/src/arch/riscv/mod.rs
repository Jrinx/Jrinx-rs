mod entry;

use jrinx_addr::VirtAddr;
use jrinx_paging::{GenericPagePerm, PagePerm};
use riscv::register::{
    scause::{Exception, Interrupt},
    sstatus::{FS, SPP},
    utvec::TrapMode,
};

use crate::{breakpoint, soft_int, timer_int, GenericContext, TrapReason};

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

impl GenericContext for Context {
    fn trap_reason(&self) -> TrapReason {
        let is_interrupt = (self.scause & (1 << (usize::BITS - 1))) != 0;
        if is_interrupt {
            let code = self.scause & !(1 << (usize::BITS - 1));
            match Interrupt::from(code) {
                Interrupt::SupervisorSoft => TrapReason::SoftwareInterrupt,
                Interrupt::SupervisorTimer => TrapReason::TimerInterrupt,
                Interrupt::SupervisorExternal => TrapReason::ExternalInterrupt,
                _ => TrapReason::Unknown { code: self.scause },
            }
        } else {
            let code = Exception::from(self.scause);
            match code {
                Exception::UserEnvCall => TrapReason::SystemCall,
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
                _ => TrapReason::Unknown { code: self.scause },
            }
        }
    }

    fn user_setup(&mut self, entry_point: usize, stack_top: usize) {
        self.regs.sp = stack_top;
        self.sstatus = 1 << 18 | (FS::Initial as usize) << 13 | (SPP::User as usize) << 8 | 1 << 5; // sum | fs | spp | spie
        self.sepc = entry_point;
        self.enable_int();
    }

    fn enable_int(&mut self) {
        self.sie = 1 << 9 | 1 << 5 | 1 << 1; // external int | timer int | software int
    }

    fn disable_int(&mut self) {
        self.sie = 0;
    }

    fn pc_advance(&mut self) {
        let is_rvc = (unsafe { core::ptr::read_volatile(self.sepc as *const u8) & 0b11 }) != 0b11;
        if is_rvc {
            self.sepc += 2;
        } else {
            self.sepc += 4;
        }
    }

    fn syscall_num(&self) -> usize {
        self.regs.a7
    }

    fn run(&mut self) {
        extern "C" {
            fn run_user(ctx: &mut Context);
        }
        unsafe { run_user(self) };
    }
}

pub(crate) fn init() {
    extern "C" {
        fn trap_entry();
    }
    unsafe {
        riscv::register::sscratch::write(0);
        riscv::register::stvec::write(trap_entry as usize, TrapMode::Direct);
    }
}

extern "C" fn handle_kern_trap(ctx: &mut Context) {
    let reason = ctx.trap_reason();
    match reason {
        TrapReason::Breakpoint { addr: _ } => breakpoint::handle(ctx),
        TrapReason::SoftwareInterrupt => soft_int::handle(ctx),
        TrapReason::TimerInterrupt => timer_int::handle(ctx),
        _ => unimplemented!(),
    }
}
