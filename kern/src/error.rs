#[derive(Debug)]
pub enum InternalError {
    DevProbeError,
    NotEnoughMem,
    InvalidVirtAddr,
    DuplicateTaskId,
}

pub type Result<T> = core::result::Result<T, InternalError>;

pub enum HaltReason {
    NormalExit,
    SysFailure,
}
