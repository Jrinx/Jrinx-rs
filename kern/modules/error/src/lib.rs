#![no_std]

#[derive(Debug)]
pub enum InternalError {
    RepeatInitialization,
    DevProbeError,
    ElfParseError,
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
    InvalidRuntimeSchedTable,
    DuplicateRuntimeSchedTable,
    InvalidTimedEventStatus,
    InvalidApexName,
    InvalidApexPriority,
    InvalidSyscallNumber,
}

pub type Result<T> = core::result::Result<T, InternalError>;
