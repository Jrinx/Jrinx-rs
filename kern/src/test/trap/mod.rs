pub(super) mod breakpoint {
    use crate::{arch, test::test_define, trap::breakpoint};

    test_define!("trap::breakpoint" => test);
    fn test() {
        for i in 0..10 {
            assert_eq!(breakpoint::count(), i);
            arch::breakpoint();
            assert_eq!(breakpoint::count(), i + 1);
        }
    }
}
