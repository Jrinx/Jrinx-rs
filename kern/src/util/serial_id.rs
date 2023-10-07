pub struct SerialIdGenerator {
    value: u64,
}

impl SerialIdGenerator {
    pub const fn new() -> Self {
        Self { value: 0 }
    }

    pub fn generate(&mut self) -> u64 {
        self.value = self
            .value
            .checked_add(1)
            .expect("serial id (64 bit) should never overflow");

        self.value
    }
}
