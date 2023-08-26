pub mod phys;
pub mod virt;

pub(super) fn early_init() {
    phys::init();
}

pub(super) fn init() {
    virt::init();
}
