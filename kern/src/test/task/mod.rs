pub(super) mod executor {
    use alloc::vec::Vec;
    use spin::RwLock;

    use crate::{
        task::{self, TaskPriority},
        test::test_define,
    };

    static QUEUE: RwLock<Vec<i32>> = RwLock::new(Vec::new());

    fn queue_push(val: i32) {
        assert_eq!(QUEUE.writer_count(), 0);
        assert_eq!(QUEUE.reader_count(), 0);
        let guard = QUEUE.try_write();
        assert!(guard.is_some());
        guard.unwrap().push(val);
    }

    test_define!("task::executor" => test);
    fn test() {
        task::spawn(
            async {
                for i in 0..10 {
                    let priority = TaskPriority::new(i + 128);
                    let this_value = i as i32 * i as i32;
                    let prev_value = if i < 9 {
                        Some((i + 1) as i32 * (i + 1) as i32)
                    } else {
                        None
                    };
                    task::spawn(
                        async move {
                            trace!("spawned task: value = {}", this_value);
                            assert_eq!(QUEUE.read().last(), prev_value.as_ref());
                            queue_push(this_value);
                        },
                        priority,
                    );
                }
                task::yield_now().await;
                assert_eq!(QUEUE.read().len(), 10);
            },
            TaskPriority::new(127),
        );
    }
}
