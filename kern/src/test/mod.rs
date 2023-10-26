mod heap;
mod mm;
mod task;
mod time;
mod trap;

use jrinx_testdef::TestDef;

use crate::conf;

pub fn find(test: &str) -> Option<fn()> {
    unsafe { TestDef::iter(conf::layout::_stest(), conf::layout::_etest()) }.find_map(|test_def| {
        if test_def.name_match(test) {
            Some(test_def.test())
        } else {
            None
        }
    })
}

pub fn all() -> impl Iterator<Item = &'static str> {
    unsafe { TestDef::iter(conf::layout::_stest(), conf::layout::_etest()) }
        .map(|test_def| test_def.name())
}
