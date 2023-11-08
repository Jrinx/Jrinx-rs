use crate::local_area_base;

pub fn set_local_pointer(cpu_id: usize) {
    let pointer = local_area_base(cpu_id);
    unsafe {
        core::arch::asm!(
            "mv gp, {}",
            in(reg) pointer,
        );
    }
}

pub fn get_local_pointer() -> usize {
    let pointer;
    unsafe {
        core::arch::asm!(
            "mv {}, gp",
            out(reg) pointer,
        );
    }
    pointer
}
