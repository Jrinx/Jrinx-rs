pub const EXECUTOR_STACK_LIMIT: usize = 0xF0000000;
pub const EXECUTOR_STACK_BOTTOM: usize = 0xE0000000;
pub const EXECUTOR_STACK_SIZE: usize = jrinx_config::PAGE_SIZE * 1024;
