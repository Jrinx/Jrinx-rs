#[derive(Debug)]
pub enum InternalError {
    DevProbeError,
    NotEnoughMem,
    InvalidCpuId,
    InvalidVirtAddr,
    DuplicateTaskId,
    InvalidExecutorId,
    DuplicateExecutorId,
    InvalidRuntimeStatus,
}

pub type Result<T> = core::result::Result<T, InternalError>;

pub enum HaltReason {
    NormalExit,
    SysFailure,
}
