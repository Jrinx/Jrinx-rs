pub(super) mod round_robin {
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

        fn order_push(inspector_order: u8, executor_order: u8) {
            ORDER.lock().push((inspector_order, executor_order));
        }

        const INSPECTOR_MAX: u8 = 3;
        const EXECUTOR_MAX: u8 = 4;

        for i in 1..=INSPECTOR_MAX {
            let inspector = Inspector::new(Executor::new(
                ExecutorPriority::default(),
                Task::new(async {}, TaskPriority::default()),
            ));

            for j in 1..=EXECUTOR_MAX {
                let inspector_order = i;
                let executor_order = j;

                inspector
                    .register(Executor::new(
                        ExecutorPriority::new(j),
                        Task::new(
                            async move {
                                trace!(
                                    "spawned task: inspector = {:?}, executor = {:?}",
                                    inspector_order,
                                    executor_order
                                );

                                order_push(inspector_order, executor_order);
                            },
                            TaskPriority::default(),
                        ),
                    ))
                    .unwrap();
            }

            Runtime::with_current(|rt| rt.register(inspector).unwrap());
        }

        Inspector::with_current(|is| is.mark_pending().unwrap()).unwrap();
        Runtime::switch_yield();

        assert!(!ORDER.is_locked());

        let order = ORDER.lock();
        assert_eq!(order.len(), (INSPECTOR_MAX * EXECUTOR_MAX) as usize);

        for i in 1..=INSPECTOR_MAX {
            for j in 1..=EXECUTOR_MAX {
                let index = ((i - 1) * EXECUTOR_MAX + (j - 1)) as usize;
                assert_eq!(order[index], (i, EXECUTOR_MAX - j + 1));
            }
        }
    }
}

pub(super) mod sched_table {
    use core::time::Duration;

    use alloc::vec::Vec;
    use jrinx_hal::{Cpu, Hal, Interrupt};
    use jrinx_multitask::{
        executor::{Executor, ExecutorPriority},
        inspector::{Inspector, InspectorId},
        runtime::{Runtime, RuntimeSchedTable, RuntimeSchedTableEntry},
        Task, TaskPriority,
    };
    use jrinx_testdef::testdef;
    use spin::Mutex;

    static TIME_BASE: Mutex<Duration> = Mutex::new(Duration::from_secs(0));

    fn set_time_datum(time: Duration) {
        *TIME_BASE.lock() = time;
    }

    fn get_time_datum() -> Duration {
        *TIME_BASE.lock()
    }

    static RECORD: Mutex<Vec<(Duration, InspectorId)>> = Mutex::new(Vec::new());

    fn record() {
        let time = hal!().cpu().get_time();
        let inspector_id = Inspector::with_current(|is| is.id()).unwrap();
        let mut record = RECORD.lock();
        let Some(&(_, last)) = record.last() else {
            record.push((time, inspector_id));
            trace!("record: time = {:?}, inspector = {:?}", time, inspector_id);
            return;
        };

        if inspector_id != last {
            record.push((time, inspector_id));
            trace!("record: time = {:?}, inspector = {:?}", time, inspector_id);
        }
    }

    #[testdef]
    fn test() {
        const INSPECTOR_MAX: usize = 3;
        const FRAME_SIZE: usize = 4;

        let mut inspector_list = Vec::with_capacity(INSPECTOR_MAX);

        for i in 1..=INSPECTOR_MAX {
            let inspector_order = i;

            let inspector = Inspector::new(Executor::new(
                ExecutorPriority::default(),
                Task::new(
                    async move {
                        trace!("spawned task: inspector = {:?}", inspector_order);
                        loop {
                            hal!().interrupt().with_saved_off(|| {
                                record();
                            });

                            core::hint::spin_loop();

                            if hal!().cpu().get_time() - get_time_datum()
                                > Duration::from_secs(2 * FRAME_SIZE as u64)
                                    + Duration::from_micros(500)
                            {
                                let _ = Runtime::with_current(|rt| rt.revoke_sched_table());
                                break;
                            }
                        }
                    },
                    TaskPriority::default(),
                ),
            ));

            inspector_list.push(inspector.id());

            Runtime::with_current(|rt| rt.register(inspector).unwrap());
        }

        let sched_table = RuntimeSchedTable::new(
            Duration::from_secs(FRAME_SIZE as _),
            [
                RuntimeSchedTableEntry {
                    inspector_id: inspector_list[0],
                    offset: Duration::from_secs(0),
                    duration: Duration::from_secs(1),
                    period: Duration::from_secs(2),
                },
                RuntimeSchedTableEntry {
                    inspector_id: inspector_list[1],
                    offset: Duration::from_secs(1),
                    duration: Duration::from_secs(1),
                    period: Duration::from_secs(4),
                },
                RuntimeSchedTableEntry {
                    inspector_id: inspector_list[0],
                    offset: Duration::from_secs(2),
                    duration: Duration::from_secs(1),
                    period: Duration::from_secs(2),
                },
                RuntimeSchedTableEntry {
                    inspector_id: inspector_list[2],
                    offset: Duration::from_secs(3),
                    duration: Duration::from_secs(1),
                    period: Duration::from_secs(4),
                },
            ]
            .into_iter(),
        )
        .unwrap();

        set_time_datum(hal!().cpu().get_time());
        trace!("time datum: {:?}", get_time_datum());

        Runtime::with_current(|rt| rt.enact_sched_table(sched_table).unwrap());
        Inspector::with_current(|is| is.mark_pending().unwrap()).unwrap();
        Runtime::switch_yield();
    }
}
