#![no_std]

#[derive(Debug)]
pub enum InternalError {
    DevProbeError,
    NotEnoughMem,
    InvalidCpuId,
    InvalidVirtAddr,
    DuplicateTaskId,
    InvalidExecutorId,
    DuplicateExecutorId,
    InvalidInspectorId,
    DuplicateInspectorId,
    InvalidInspectorStatus,
    InvalidRuntimeStatus,
    InvalidTimedEventStatus,
}

pub type Result<T> = core::result::Result<T, InternalError>;

pub enum HaltReason {
    NormalExit,
    SysFailure,
}
