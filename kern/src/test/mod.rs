mod heap;
mod mm;
mod trap;

use core::mem;

use crate::conf;

pub(in crate::test) struct TestDef {
    name: &'static str,
    test: fn(),
}

macro_rules! test_define {
    ($name:literal => $test:ident) => {
        #[used(linker)]
        #[link_section = concat!(".test.", $name)]
        static TEST_DEF: &$crate::test::TestDef = &$crate::test::TestDef {
            name: $name,
            test: $test,
        };
    };
}
pub(crate) use test_define;

pub fn find(test: &str) -> Option<fn()> {
    (conf::layout::_stest()..conf::layout::_etest())
        .step_by(mem::size_of::<&TestDef>())
        .map(|a| unsafe { *(a as *const &TestDef) })
        .find_map(|test_def| {
            if test_def.name == test {
                Some(test_def.test)
            } else {
                None
            }
        })
}
