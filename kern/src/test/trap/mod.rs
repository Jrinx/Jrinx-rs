pub(super) mod breakpoint {
    use crate::{arch, trap::breakpoint::get_breakpoint_count};

    pub(in crate::test) fn test() {
        for i in 0..10 {
            assert_eq!(get_breakpoint_count(), i);
            arch::trap::raise_breakpoint();
            assert_eq!(get_breakpoint_count(), i + 1);
        }
    }
}
