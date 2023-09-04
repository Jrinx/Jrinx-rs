use spin::Mutex;

use crate::info;

static SEED: Mutex<usize> = Mutex::new(0xdeadbeef);

pub(super) fn init() {
    if let Some(seed) = option_env!("RAND_SEED") {
        let mut x = SEED.lock();
        let seed: usize = seed.parse().unwrap();
        info!("pseudo random seed: {}", seed);
        *x = seed;
    }
}

pub fn rand() -> usize {
    let mut x = SEED.lock();
    *x ^= *x << 13;
    *x ^= *x >> 17;
    *x ^= *x << 5;
    *x
}
