#[derive(Debug)]
pub enum InternalError {
    DevProbeError,
    NotEnoughMem,
    InvalidVirtAddr,
}

pub type Result<T> = core::result::Result<T, InternalError>;

pub enum HaltReason {
    NormalExit,
    SysFailure,
}
