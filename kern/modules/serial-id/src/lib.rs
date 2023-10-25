#![no_std]

pub trait SerialIdGenerator: Clone + Copy + Eq + PartialEq + Ord + PartialOrd {
    fn generate() -> Self;
}
