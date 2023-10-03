use crate::conf;

pub const EXECUTOR_STACK_LIMIT: usize = 0xFFFFFFF000000000;
pub const EXECUTOR_STACK_SIZE: usize = conf::PAGE_SIZE * 1024;
