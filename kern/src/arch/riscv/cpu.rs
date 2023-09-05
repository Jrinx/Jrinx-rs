pub fn id() -> usize {
    let id: usize;
    unsafe {
        core::arch::asm!(
            "mv {}, tp",
            out(reg) id,
        );
    }
    id
}

pub fn time() -> usize {
    riscv::register::time::read()
}
