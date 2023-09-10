mod heap;
mod mm;
mod trap;

static TEST_CASES: &[(&'static str, fn())] = &[
    ("heap", heap::test),
    ("mm::phys", mm::phys::test),
    ("mm::virt", mm::virt::test),
    ("trap::breakpoint", trap::breakpoint::test),
];

pub fn find(test: &str) -> Option<fn()> {
    TEST_CASES
        .iter()
        .find(|(name, _)| *name == test)
        .map(|(_, test)| *test)
}
