use alloc::vec;
use jrinx_testdef_macro::testdef;

#[testdef]
fn test() {
    let mut v = vec![1, 2, 3];
    assert_eq!(v.len(), 3);
    for i in 4..10 {
        v.push(i);
    }
    for (i, &item) in v.iter().enumerate().take(9) {
        assert_eq!(item, i + 1);
    }
}
