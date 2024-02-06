use crate::bindings::*;

pub type ApexProcessName = ApexName;
pub type ApexProcessIndex = ApexInteger;
pub type ApexProcessId = ApexLongInteger;

#[repr(u32)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ApexProcessState {
    #[default]
    Dormant = 0,
    Ready = 1,
    Running = 2,
    Waiting = 3,
    Faulted = 4,
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ApexProcessAttribute {
    pub period: ApexSystemTime,
    pub time_capacity: ApexSystemTime,
    pub entry_point: ApexSystemAddress,
    pub stack_size: ApexStackSize,
    pub base_priority: ApexPriority,
    pub deadline: ApexDeadline,
    pub name: ApexProcessName,
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ApexProcessStatus {
    pub deadline_time: ApexSystemTime,
    pub current_priority: ApexPriority,
    pub process_state: ApexProcessState,
    pub attributes: ApexProcessAttribute,
}

pub trait ApexProcessService {
    fn get_process_id(
        &self,
        process_name: &ApexProcessName,
    ) -> Result<ApexProcessId, ApexReturnCode>;

    fn get_process_status(
        &self,
        process_id: ApexProcessId,
    ) -> Result<ApexProcessStatus, ApexReturnCode>;

    fn create_process(
        &self,
        attributes: &ApexProcessAttribute,
    ) -> Result<ApexProcessId, ApexReturnCode>;

    fn set_priority(
        &self,
        process_id: ApexProcessId,
        priority: ApexPriority,
    ) -> Result<(), ApexReturnCode>;

    fn suspend_self(&self, time_out: ApexSystemTime) -> Result<(), ApexReturnCode>;

    fn suspend(&self, process_id: ApexProcessId) -> Result<(), ApexReturnCode>;

    fn resume(&self, process_id: ApexProcessId) -> Result<(), ApexReturnCode>;

    fn stop_self(&self) -> !;

    fn stop(&self, process_id: ApexProcessId) -> Result<(), ApexReturnCode>;

    fn start(&self, process_id: ApexProcessId) -> Result<(), ApexReturnCode>;

    fn delayed_start(
        &self,
        process_id: ApexProcessId,
        start_time: ApexSystemTime,
    ) -> Result<(), ApexReturnCode>;

    fn lock_preemption(&self) -> Result<ApexLockLevel, ApexReturnCode>;

    fn unlock_preemption(&self) -> Result<ApexLockLevel, ApexReturnCode>;

    fn get_my_id(&self) -> Result<ApexProcessId, ApexReturnCode>;

    fn initialize_process_core_affinity(
        &self,
        process_id: ApexProcessId,
        process_core_id: ApexProcessorCoreId,
    ) -> Result<(), ApexReturnCode>;

    fn get_my_processor_core_id(&self) -> Result<ApexProcessorCoreId, ApexReturnCode>;

    fn get_my_index(&self) -> Result<ApexProcessIndex, ApexReturnCode>;
}
