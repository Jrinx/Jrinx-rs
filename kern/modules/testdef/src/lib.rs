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

    pub fn name(&self) -> &'static str {
        self.name.strip_prefix("jrinx::test::").unwrap()
    }

    pub fn test(&self) -> fn() {
        self.test
    }

    pub unsafe fn iter(start: usize, end: usize) -> impl Iterator<Item = &'static TestDef> {
        (start..end)
            .step_by(core::mem::size_of::<&TestDef>())
            .map(|a| *(a as *const &TestDef))
    }
}
