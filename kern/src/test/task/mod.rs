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

pub(super) mod runtime {
    use alloc::vec::Vec;
    use spin::Mutex;

    use crate::{
        cpudata,
        task::{
            executor::{Executor, ExecutorBehaviorOnNoTask, ExecutorPriority},
            runtime, Task, TaskPriority,
        },
        test::test_define,
    };

    test_define!("task::runtime" => test);
    fn test() {
        static ORDER: Mutex<Vec<(u16, u16)>> = Mutex::new(Vec::new());

        fn order_push(executor_order: u16, task_order: u16) {
            ORDER.lock().push((executor_order, task_order));
        }

        fn order_last() -> Option<(u16, u16)> {
            ORDER.lock().last().cloned()
        }

        const EXECUTOR_MAX: u16 = 5;
        const TASK_MAX: u16 = 5;
        const EXECUTOR_PRIORITY: ExecutorPriority = ExecutorPriority::new(1);

        assert!(EXECUTOR_PRIORITY > cpudata::with_cpu_executor(|executor| executor.priority()));

        for i in 1..=EXECUTOR_MAX {
            let mut executor = Executor::new(EXECUTOR_PRIORITY, ExecutorBehaviorOnNoTask::EXIT);

            for j in 1..=TASK_MAX {
                let executor_order = i;
                let task_order = j;

                executor
                    .spawn(Task::new(
                        async move {
                            let prev_order = order_last();

                            if executor_order == 1 && task_order == TASK_MAX {
                                assert_eq!(prev_order, None);
                            } else if executor_order == 1 {
                                assert_eq!(prev_order, Some((EXECUTOR_MAX, task_order + 1)));
                            } else {
                                assert_eq!(prev_order, Some((executor_order - 1, task_order)));
                            }

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

            cpudata::with_cpu_runtime(|rt| rt.register_executor(executor)).unwrap();
        }

        runtime::switch_yield();

        assert_eq!(ORDER.is_locked(), false);

        let order = ORDER.lock();
        assert_eq!(order.len(), (EXECUTOR_MAX * TASK_MAX) as usize);

        for i in 1..=TASK_MAX {
            for j in 1..=EXECUTOR_MAX {
                let index = ((i - 1) * EXECUTOR_MAX + (j - 1)) as usize;
                assert_eq!(order[index], (j, TASK_MAX - i + 1));
            }
        }
    }
}
