use jrinx_abi::sysfn::*;
use jrinx_apex::*;

pub struct Process;

impl ApexProcessService for Process {
    fn get_process_id(
        &self,
        process_name: &ApexProcessName,
    ) -> Result<ApexProcessId, ApexReturnCode> {
        let mut id = ApexProcessId::default();
        sys_get_process_id(process_name, &mut id).as_result(id)
    }

    fn get_process_status(
        &self,
        process_id: ApexProcessId,
    ) -> Result<ApexProcessStatus, ApexReturnCode> {
        let mut status = ApexProcessStatus::default();
        sys_get_process_status(process_id, &mut status).as_result(status)
    }

    fn create_process(
        &self,
        attributes: &ApexProcessAttribute,
    ) -> Result<ApexProcessId, ApexReturnCode> {
        let mut id = ApexProcessId::default();
        sys_create_process(attributes, &mut id).as_result(id)
    }

    fn set_priority(
        &self,
        _process_id: ApexProcessId,
        _priority: ApexPriority,
    ) -> Result<(), ApexReturnCode> {
        todo!()
    }

    fn suspend_self(&self, _time_out: ApexSystemTime) -> Result<(), ApexReturnCode> {
        todo!()
    }

    fn suspend(&self, _process_id: ApexProcessId) -> Result<(), ApexReturnCode> {
        todo!()
    }

    fn resume(&self, _process_id: ApexProcessId) -> Result<(), ApexReturnCode> {
        todo!()
    }

    fn stop_self(&self) -> ! {
        todo!()
    }

    fn stop(&self, _process_id: ApexProcessId) -> Result<(), ApexReturnCode> {
        todo!()
    }

    fn start(&self, process_id: ApexProcessId) -> Result<(), ApexReturnCode> {
        sys_start(process_id).into()
    }

    fn delayed_start(
        &self,
        _process_id: ApexProcessId,
        _delay_time: ApexSystemTime,
    ) -> Result<(), ApexReturnCode> {
        todo!()
    }

    fn lock_preemption(&self) -> Result<ApexLockLevel, ApexReturnCode> {
        todo!()
    }

    fn unlock_preemption(&self) -> Result<ApexLockLevel, ApexReturnCode> {
        todo!()
    }

    fn get_my_id(&self) -> Result<ApexProcessId, ApexReturnCode> {
        todo!()
    }

    fn initialize_process_core_affinity(
        &self,
        process_id: ApexProcessId,
        processor_core_id: ApexProcessorCoreId,
    ) -> Result<(), ApexReturnCode> {
        sys_initialize_process_core_affinity(process_id, processor_core_id).into()
    }

    fn get_my_processor_core_id(&self) -> Result<ApexProcessorCoreId, ApexReturnCode> {
        todo!()
    }

    fn get_my_index(&self) -> Result<ApexProcessIndex, ApexReturnCode> {
        todo!()
    }
}
