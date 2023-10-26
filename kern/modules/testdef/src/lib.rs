#![no_std]

#[repr(C)]
pub struct TestDef {
    name: &'static str,
    test: fn(),
}

impl TestDef {
    pub const fn new(name: &'static str, test: fn()) -> Self {
        Self { name, test }
    }
}

pub fn all() -> impl Iterator<Item = &'static str> {
    unsafe { testdef_iter() }.map(|test_def| test_def.name)
}

pub fn find(name: &str) -> Option<fn()> {
    unsafe {
        testdef_iter().find_map(|test_def| test_def.name.contains(name).then_some(test_def.test))
    }
}

unsafe fn testdef_iter() -> impl Iterator<Item = &'static TestDef> {
    (jrinx_layout::_stest()..jrinx_layout::_etest())
        .step_by(core::mem::size_of::<&TestDef>())
        .map(|a| *(a as *const &TestDef))
}
