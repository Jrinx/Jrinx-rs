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
        self.name
    }

    pub fn name_match(&self, s: &str) -> bool {
        self.name.contains(s)
    }

    pub fn test(&self) -> fn() {
        self.test
    }

    /// # Safety
    ///
    /// This function is safe as long as the caller ensures that the [`start`, `end`) is a valid
    /// contiguous range of pointers to [`TestDef`].
    pub unsafe fn iter(start: usize, end: usize) -> impl Iterator<Item = &'static TestDef> {
        (start..end)
            .step_by(core::mem::size_of::<&TestDef>())
            .map(|a| *(a as *const &TestDef))
    }
}
