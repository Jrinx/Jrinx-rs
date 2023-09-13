use core::fmt::Debug;

use cfg_if::cfg_if;

use crate::trap::TrapReason;

cfg_if! {
    if #[cfg(any(target_arch = "riscv64", target_arch = "riscv32"))] {
        mod riscv;
        pub use riscv::*;
    } else {
        compile_error!("unsupported target_arch");
    }
}

pub trait AbstractContext: Debug + Default + Clone + Copy {
    fn trap_reason(&self) -> TrapReason;

    fn setup_user(&mut self, entry_point: usize, stack_top: usize);

    fn acc_pc(&mut self);

    fn get_syscall_num(&self) -> usize;

    fn set_syscall_ret(&mut self, ret: usize);

    fn run(&mut self);
}
