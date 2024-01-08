pub(super) mod executor {
    use alloc::vec::Vec;
    use jrinx_multitask::{spawn, yield_now, TaskPriority};
    use jrinx_testdef::testdef;
    use spin::RwLock;

    static QUEUE: RwLock<Vec<i32>> = RwLock::new(Vec::new());

    fn queue_push(val: i32) {
        assert_eq!(QUEUE.writer_count(), 0);
        assert_eq!(QUEUE.reader_count(), 0);
        let guard = QUEUE.try_write();
        assert!(guard.is_some());
        guard.unwrap().push(val);
    }

    #[testdef]
    fn test() {
        spawn!(
            pri := TaskPriority::MAX / 3 => async {
            for i in 0..10 {
                let priority = TaskPriority::new(i + TaskPriority::MAX / 2);
                let this_value = i as i32 * i as i32;
                let prev_value = if i < 9 {
                    Some((i + 1) as i32 * (i + 1) as i32)
                } else {
                    None
                };
                spawn!(
                    pri := priority => async move {
                        trace!("spawned task: value = {}", this_value);
                        assert_eq!(QUEUE.read().last(), prev_value.as_ref());
                        queue_push(this_value);
                    }
                );
            }
            yield_now!();
            assert_eq!(QUEUE.read().len(), 10);
        });
    }
}

pub(super) mod inspector {
    use alloc::vec::Vec;
    use jrinx_multitask::{
        executor::{Executor, ExecutorPriority},
        inspector::Inspector,
        runtime::Runtime,
        Task, TaskPriority,
    };
    use jrinx_testdef::testdef;
    use spin::Mutex;

    #[testdef]
    fn test() {
        static ORDER: Mutex<Vec<(u8, u8)>> = Mutex::new(Vec::new());

        fn order_push(executor_order: u8, task_order: u8) {
            ORDER.lock().push((executor_order, task_order));
        }

        const EXECUTOR_MAX: u8 = 4;
        const TASK_MAX: u8 = 5;

        for i in 1..=EXECUTOR_MAX {
            let mut executor = Executor::new(
                ExecutorPriority::new(i),
                Task::new(async {}, TaskPriority::default()),
            );

            for j in 1..=TASK_MAX {
                let executor_order = i;
                let task_order = j;

                executor
                    .spawn(Task::new(
                        async move {
                            trace!(
                                "spawned task: executor = {:?}, task order = {:?}",
                                executor_order,
                                task_order
                            );

                            order_push(executor_order, task_order);

                            Runtime::switch_yield();
                        },
                        TaskPriority::new(j),
                    ))
                    .unwrap();
            }

            Inspector::with_current(|is| is.register(executor).unwrap()).unwrap();
        }

        Runtime::switch_yield();

        assert!(!ORDER.is_locked());

        let order = ORDER.lock();
        assert_eq!(order.len(), (EXECUTOR_MAX * TASK_MAX) as usize);

        for i in 1..=EXECUTOR_MAX {
            for j in 1..=TASK_MAX {
                let index = ((i - 1) * TASK_MAX + (j - 1)) as usize;
                assert_eq!(order[index], (EXECUTOR_MAX - i + 1, TASK_MAX - j + 1));
            }
        }
    }
}

pub(super) mod runtime;
