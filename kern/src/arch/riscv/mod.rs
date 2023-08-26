#[cfg(target_arch = "riscv32")]
mod riscv32;
#[cfg(target_arch = "riscv32")]
pub use riscv32::*;

#[cfg(target_arch = "riscv64")]
mod riscv64;
#[cfg(target_arch = "riscv64")]
pub use riscv64::*;

#[cfg_attr(feature = "board_virt", path = "board/virt.rs")]
mod board;
