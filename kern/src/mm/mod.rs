pub mod phys;
pub mod virt;

pub(super) fn init() {
    virt::init();
}
