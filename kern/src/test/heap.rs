use alloc::vec;

use super::test_define;

test_define!("heap" => test);
fn test() {
    let mut v = vec![1, 2, 3];
    assert_eq!(v.len(), 3);
    for i in 4..10 {
        v.push(i);
    }
    for i in 0..9 {
        assert_eq!(v[i], i + 1);
    }
}
