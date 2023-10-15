pub(super) mod status {
    use core::time::Duration;

    use spin::Mutex;

    use crate::{
        arch, cpudata,
        test::test_define,
        time::{TimedEvent, TimedEventStatus},
    };

    test_define!("time::status" => test);
    fn test() {
        static DATA: Mutex<Option<TimedEventStatus>> = Mutex::new(None);

        let timed_event = TimedEvent::new(
            arch::cpu::time() + Duration::from_secs(1),
            || {
                *DATA.lock() = Some(TimedEventStatus::Timeout);
            },
            || {
                *DATA.lock() = Some(TimedEventStatus::Cancelled);
            },
        );

        cpudata::with_cpu_timed_event_queue(|queue| queue.add(timed_event.clone())).unwrap();

        arch::wait_for_interrupt();

        assert_eq!(*DATA.lock(), Some(TimedEventStatus::Timeout));

        let timed_event = TimedEvent::new(
            arch::cpu::time() + Duration::from_secs(1),
            || {
                *DATA.lock() = Some(TimedEventStatus::Timeout);
            },
            || {
                *DATA.lock() = Some(TimedEventStatus::Cancelled);
            },
        );

        let timed_event_id = { timed_event.lock().id() };

        cpudata::with_cpu_timed_event_queue(|queue| queue.add(timed_event.clone())).unwrap();

        cpudata::with_cpu_timed_event_queue(|queue| queue.cancel(timed_event_id).unwrap()).unwrap();

        assert_eq!(*DATA.lock(), Some(TimedEventStatus::Cancelled));
        assert_eq!(
            cpudata::with_cpu_timed_event_queue(|queue| queue.peek()).unwrap(),
            None
        );
    }
}

pub(super) mod queue {
    use core::time::Duration;

    use alloc::vec::Vec;
    use spin::Mutex;

    use crate::{arch, cpudata, test::test_define, time::TimedEvent};

    test_define!("time::queue" => test);
    fn test() {
        const EVENT_MAX: usize = 3;
        static ORDER: Mutex<Vec<Duration>> = Mutex::new(Vec::new());

        fn order_push(time: Duration) {
            ORDER.lock().push(time);
        }

        for i in (1..=EVENT_MAX).rev() {
            let timed_event = TimedEvent::new(
                arch::cpu::time() + Duration::from_secs(i as u64),
                || {
                    order_push(arch::cpu::time());
                },
                || {
                    order_push(arch::cpu::time());
                },
            );

            cpudata::with_cpu_timed_event_queue(|queue| queue.add(timed_event)).unwrap();
        }

        for i in 1..=EVENT_MAX {
            arch::wait_for_interrupt();
            assert_eq!(ORDER.lock().len(), i);
        }

        let order = ORDER.lock();
        for i in 0..EVENT_MAX - 1 {
            assert!(order[i] < order[i + 1]);
        }
    }
}
