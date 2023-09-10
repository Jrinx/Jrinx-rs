use crate::trap::breakpoint::do_breakpoint;

pub(super) fn handle_breakpoint(sepc: &mut usize) {
    let addr = *sepc;
    let rvc = unsafe { core::ptr::read_volatile(addr as *const u8) } & 0b11 != 0b11;
    do_breakpoint(addr);
    *sepc += if rvc { 2 } else { 4 };
}

pub fn raise_breakpoint() {
    unsafe {
        riscv::asm::ebreak();
    }
}
