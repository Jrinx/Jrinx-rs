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

pub(super) mod inspector {
    use alloc::vec::Vec;
    use spin::Mutex;

    use crate::{
        cpudata,
        task::{
            executor::{Executor, ExecutorPriority},
            runtime, Task, TaskPriority,
        },
        test::test_define,
    };

    test_define!("task::inspector" => test);
    fn test() {
        static ORDER: Mutex<Vec<(u16, u16)>> = Mutex::new(Vec::new());

        fn order_push(executor_order: u16, task_order: u16) {
            ORDER.lock().push((executor_order, task_order));
        }

        const EXECUTOR_MAX: u16 = 4;
        const TASK_MAX: u16 = 5;

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

                            runtime::switch_yield();
                        },
                        TaskPriority::new(j),
                    ))
                    .unwrap();
            }

            cpudata::with_cpu_inspector(|inspector| inspector.register_executor(executor).unwrap())
                .unwrap();
        }

        runtime::switch_yield();

        assert_eq!(ORDER.is_locked(), false);

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
