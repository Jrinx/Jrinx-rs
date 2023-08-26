mod heap;
mod mm;

static TEST_CASES: &[(&'static str, fn())] = &[
    ("heap", heap::test),
    ("mm::phys", mm::phys::test),
    ("mm::virt", mm::virt::test),
];

pub fn find(test: &str) -> Option<fn()> {
    TEST_CASES
        .iter()
        .find(|(name, _)| *name == test)
        .map(|(_, test)| *test)
}
