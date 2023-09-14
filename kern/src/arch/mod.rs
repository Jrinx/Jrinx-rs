use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(any(target_arch = "riscv64", target_arch = "riscv32"))] {
        mod riscv;
        pub use riscv::*;
    } else {
        compile_error!("unsupported target_arch");
    }
}
