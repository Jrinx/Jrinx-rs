mod heap;
mod mm;
mod task;
mod time;
mod trap;

use jrinx_testdef::TestDef;

pub fn find(test: &str) -> Option<fn()> {
    unsafe { TestDef::iter(jrinx_layout::_stest(), jrinx_layout::_etest()) }.find_map(|test_def| {
        if test_def.name_match(test) {
            Some(test_def.test())
        } else {
            None
        }
    })
}

pub fn all() -> impl Iterator<Item = &'static str> {
    unsafe { TestDef::iter(jrinx_layout::_stest(), jrinx_layout::_etest()) }
        .map(|test_def| test_def.name())
}
