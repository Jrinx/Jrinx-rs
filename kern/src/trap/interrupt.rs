use crate::arch;

pub fn with_interrupt_saved_off<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    let int = arch::int_is_enabled();
    arch::int_disable();

    let result = f();

    if int {
        arch::int_enable();
    }

    result
}
