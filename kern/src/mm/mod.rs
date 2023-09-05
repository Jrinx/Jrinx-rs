pub mod phys;
pub mod virt;

pub(super) fn early_init() {
    phys::early_init();
}

pub(super) fn init() {
    phys::init();
    virt::init();
}
