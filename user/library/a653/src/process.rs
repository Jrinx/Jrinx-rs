use a653rs::bindings::*;
use jrlib_sys::*;

pub struct Process;

impl ApexProcessP1 for Process {
    fn get_process_id(process_name: ProcessName) -> Result<ProcessId, ErrorReturnCode> {
        let mut id = ProcessId::default();
        ErrorReturnCode::from(sys_get_process_id(&process_name, &mut id)).map(|_| id)
    }

    fn get_process_status(process_id: ProcessId) -> Result<ApexProcessStatus, ErrorReturnCode> {
        let mut status = ApexProcessStatus {
            attributes: ApexProcessAttribute {
                period: ApexSystemTime::default(),
                time_capacity: ApexSystemTime::default(),
                entry_point:
                // Safety: This is a initialization value and will be overwritten by the kernel.
                unsafe {
                    #[allow(invalid_value)]
                    core::mem::transmute(0usize)
                },
                stack_size: StackSize::default(),
                base_priority: Priority::default(),
                deadline: Deadline::Soft,
                name: ProcessName::default(),
            },
            current_priority: Priority::default(),
            deadline_time: ApexSystemTime::default(),
            process_state: ProcessState::Dormant,
        };
        ErrorReturnCode::from(sys_get_process_status(process_id, &mut status)).map(|_| status)
    }

    fn set_priority(_process_id: ProcessId, _priority: Priority) -> Result<(), ErrorReturnCode> {
        todo!()
    }

    fn suspend_self(_time_out: ApexSystemTime) -> Result<(), ErrorReturnCode> {
        todo!()
    }

    fn suspend(_process_id: ProcessId) -> Result<(), ErrorReturnCode> {
        todo!()
    }

    fn resume(_process_id: ProcessId) -> Result<(), ErrorReturnCode> {
        todo!()
    }

    fn stop_self() {
        todo!()
    }

    fn stop(_process_id: ProcessId) -> Result<(), ErrorReturnCode> {
        todo!()
    }

    fn delayed_start(
        _process_id: ProcessId,
        _delay_time: ApexSystemTime,
    ) -> Result<(), ErrorReturnCode> {
        todo!()
    }

    fn lock_preemption() -> Result<LockLevel, ErrorReturnCode> {
        todo!()
    }

    fn unlock_preemption() -> Result<LockLevel, ErrorReturnCode> {
        todo!()
    }

    fn get_my_id() -> Result<ProcessId, ErrorReturnCode> {
        todo!()
    }

    fn initialize_process_core_affinity(
        process_id: ProcessId,
        processor_core_id: ProcessorCoreId,
    ) -> Result<(), ErrorReturnCode> {
        ErrorReturnCode::from(sys_initialize_process_core_affinity(
            process_id,
            processor_core_id,
        ))
    }

    fn get_my_processor_core_id() -> ProcessorCoreId {
        todo!()
    }

    fn get_my_index() -> Result<ProcessIndex, ErrorReturnCode> {
        todo!()
    }
}

impl ApexProcessP4 for Process {
    fn create_process(attributes: &ApexProcessAttribute) -> Result<ProcessId, ErrorReturnCode> {
        let mut id = ProcessId::default();
        ErrorReturnCode::from(sys_create_process(attributes, &mut id)).map(|_| id)
    }

    fn start(process_id: ProcessId) -> Result<(), ErrorReturnCode> {
        ErrorReturnCode::from(sys_start(process_id))
    }
}
