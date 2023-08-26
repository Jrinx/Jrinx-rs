use alloc::vec;

pub(super) fn test() {
    let mut v = vec![1, 2, 3];
    assert_eq!(v.len(), 3);
    for i in 4..10 {
        v.push(i);
    }
    for i in 0..9 {
        assert_eq!(v[i], i + 1);
    }
}
