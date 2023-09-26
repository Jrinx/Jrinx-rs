pub(super) mod global_sched {
    use alloc::{format, vec::Vec};
    use spin::RwLock;

    use crate::{
        cpudata,
        task::{
            sched::{self, Scheduler},
            Task, TaskId,
        },
        test::test_define,
    };

    test_define!("task::global-sched" => test);
    fn test() {
        const MIN_TASK_ID: TaskId = 2;
        const MAX_TASK_ID: TaskId = 10;

        fn with_result(f: impl FnOnce(&mut Vec<TaskId>)) {
            static RECORD: RwLock<Vec<TaskId>> = RwLock::new(Vec::new());
            assert_eq!(RECORD.reader_count(), 0);
            assert_eq!(RECORD.writer_count(), 0);
            f(&mut *RECORD.write());
        }

        fn subtask_main(arg: usize) {
            let current_task = cpudata::get_current_task().unwrap();
            let task_name = current_task.get_name();
            let task_ident = current_task.get_ident();
            debug!("task '{}' started with arg {}", task_name, arg);
            assert_eq!(
                current_task.get_priority() * current_task.get_priority(),
                arg,
            );
            with_result(|record| {
                match record.last() {
                    Some(last) => assert_eq!(*last, task_ident + 1),
                    None => assert_eq!(task_ident, MAX_TASK_ID),
                }
                record.push(cpudata::get_current_task().unwrap().get_ident());
            });
            debug!("task '{}' ended", task_name);
        }

        for i in MIN_TASK_ID..=MAX_TASK_ID {
            let priority = (i - MIN_TASK_ID + 1) as usize;
            sched::with_global_scheduler(|scheduler| {
                scheduler.insert(
                    Task::create(
                        format!("task#{}", i).as_str(),
                        priority,
                        subtask_main as usize,
                        priority * priority,
                    )
                    .unwrap(),
                );
            })
        }

        sched::global_switch();
        with_result(|record| {
            assert_eq!(record.len(), (MAX_TASK_ID - MIN_TASK_ID + 1) as usize);
            assert_eq!(record.first(), Some(&MAX_TASK_ID));
            assert_eq!(record.last(), Some(&MIN_TASK_ID));
            assert!(record.is_sorted_by_key(|&x| u64::MAX - x));
        });
    }
}
