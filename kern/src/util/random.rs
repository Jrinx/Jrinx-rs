use spin::Mutex;

static SEED: Mutex<usize> = Mutex::new(0);

pub fn init() {
    if let Some(seed) = option_env!("RAND_SEED") {
        let seed: usize = seed.parse().unwrap();
        trace!("pseudo random seed: {}", seed);
        *SEED.lock() = seed;
    }
}

pub fn rand() -> usize {
    let mut x = SEED.lock();
    *x ^= *x << 13;
    *x ^= *x >> 17;
    *x ^= *x << 5;
    *x
}
