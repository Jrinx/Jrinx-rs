pub mod phys;
pub mod virt;

pub(super) fn init() {
    phys::init();
    virt::init();
}
