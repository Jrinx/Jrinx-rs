pub(super) mod status {
    use core::time::Duration;

    use jrinx_hal::{Hal, Interrupt};
    use jrinx_testdef::testdef;
    use spin::Mutex;

    use crate::{
        arch,
        time::{self, TimedEvent, TimedEventHandler, TimedEventStatus},
    };

    #[testdef]
    fn test() {
        static DATA: Mutex<Option<TimedEventStatus>> = Mutex::new(None);

        TimedEvent::create(
            arch::cpu::time() + Duration::from_secs(1),
            TimedEventHandler::new(
                || {
                    *DATA.lock() = Some(TimedEventStatus::Timeout);
                },
                || {
                    *DATA.lock() = Some(TimedEventStatus::Cancelled);
                },
            ),
        );

        hal!().interrupt().wait();

        assert_eq!(*DATA.lock(), Some(TimedEventStatus::Timeout));

        let tracker = TimedEvent::create(
            arch::cpu::time() + Duration::from_secs(1),
            TimedEventHandler::new(
                || {
                    *DATA.lock() = Some(TimedEventStatus::Timeout);
                },
                || {
                    *DATA.lock() = Some(TimedEventStatus::Cancelled);
                },
            ),
        );
        tracker.cancel().unwrap();

        assert_eq!(*DATA.lock(), Some(TimedEventStatus::Cancelled));
        assert!(time::with_current(|tq| tq.peek_outdated()).is_none());
    }
}

pub(super) mod queue {
    use core::time::Duration;

    use alloc::vec::Vec;
    use jrinx_hal::{Hal, Interrupt};
    use jrinx_testdef::testdef;
    use spin::Mutex;

    use crate::{
        arch,
        time::{TimedEvent, TimedEventHandler},
    };

    #[testdef]
    fn test() {
        const EVENT_MAX: usize = 3;
        static ORDER: Mutex<Vec<usize>> = Mutex::new(Vec::new());

        fn order_push(order: usize) {
            ORDER.lock().push(order);
        }

        for i in (1..=EVENT_MAX).rev() {
            let this_order = i;
            TimedEvent::create(
                arch::cpu::time() + Duration::from_secs(i as u64),
                TimedEventHandler::new(
                    move || {
                        order_push(this_order);
                    },
                    || panic!("this timed-event should not be cancelled"),
                ),
            );
        }

        for i in 1..=EVENT_MAX {
            hal!().interrupt().wait();
            assert_eq!(ORDER.lock().len(), i);
        }

        let order = ORDER.lock();
        for i in 0..EVENT_MAX {
            assert_eq!(order[i], i + 1);
        }
    }
}
