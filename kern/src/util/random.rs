static mut SEED: usize = 0xdeadbeef;

pub fn init() {
    if let Some(seed) = option_env!("RAND_SEED") {
        let x = unsafe { &mut SEED };
        let seed: usize = seed.parse().unwrap();
        info!("pseudo random seed: {}", seed);
        *x = seed;
    }
}

pub fn rand() -> usize {
    let x = unsafe { &mut SEED };
    *x ^= *x << 13;
    *x ^= *x >> 17;
    *x ^= *x << 5;
    *x
}
