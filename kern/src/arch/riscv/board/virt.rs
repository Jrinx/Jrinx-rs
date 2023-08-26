use crate::driver::serial::ns16550a;

pub(super) fn init() {
    ns16550a::init();
}
