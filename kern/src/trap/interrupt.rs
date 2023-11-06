use jrinx_hal::{Hal, Interrupt};

pub fn with_interrupt_saved_off<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    let int = hal!().interrupt().is_enabled();
    hal!().interrupt().disable();

    let result = f();

    if int {
        hal!().interrupt().enable();
    }

    result
}
