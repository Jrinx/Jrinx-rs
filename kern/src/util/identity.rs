use spin::Mutex;

pub trait IdGenerater {
    fn generate() -> u64 {
        static COUNTER: Mutex<u64> = Mutex::new(1);
        let mut counter = COUNTER.lock();
        let id = *counter;
        *counter = counter
            .checked_add(1)
            .expect("identity counter (64 bit) should never overflow");
        id
    }
}
