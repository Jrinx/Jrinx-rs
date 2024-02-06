mod macros;

use jrinx_apex::*;

use crate::sysno::*;
use macros::*;

def_sysfn! {
    @SYS_GET_PARTITION_STATUS
    sys_get_partition_status(
        status: *mut ApexPartitionStatus,
    ) -> ApexReturnCode

    @SYS_SET_PARTITION_MODE
    sys_set_partition_mode(
        mode: ApexOperatingMode,
    ) -> ApexReturnCode
}

def_sysfn! {
    @SYS_GET_PROCESS_ID
    sys_get_process_id(
        name: *const ApexProcessName,
        id: *mut ApexProcessId,
    ) -> ApexReturnCode

    @SYS_GET_PROCESS_STATUS
    sys_get_process_status(
        id: ApexProcessId,
        status: *mut ApexProcessStatus,
    ) -> ApexReturnCode

    @SYS_CREATE_PROCESS
    sys_create_process(
        attr: *const ApexProcessAttribute,
        id: *mut ApexProcessId,
    ) -> ApexReturnCode

    @SYS_SET_PRIORITY
    sys_set_priority(
        id: ApexProcessId,
        priority: ApexPriority,
    ) -> ApexReturnCode

    @SYS_SUSPEND_SELF
    sys_suspend_self(
        timeout: ApexSystemTime,
    ) -> ApexReturnCode

    @SYS_SUSPEND
    sys_suspend(
        id: ApexProcessId,
    ) -> ApexReturnCode

    @SYS_RESUME
    sys_resume(
        id: ApexProcessId,
    ) -> ApexReturnCode

    @SYS_STOP_SELF
    sys_stop_self() -> !

    @SYS_STOP
    sys_stop(
        id: ApexProcessId,
    ) -> ApexReturnCode

    @SYS_START
    sys_start(
        id: ApexProcessId,
    ) -> ApexReturnCode

    @SYS_DELAYED_START
    sys_delayed_start(
        id: ApexProcessId,
        delay_time: ApexSystemTime,
    ) -> ApexReturnCode

    @SYS_LOCK_PREEMPTION
    sys_lock_preemption(
        lock_level: ApexLockLevel,
    ) -> ApexReturnCode

    @SYS_UNLOCK_PREEMPTION
    sys_unlock_preemption(
        lock_level: *mut ApexLockLevel,
    ) -> ApexReturnCode

    @SYS_GET_MY_ID
    sys_get_my_id(
        id: *mut ApexProcessId,
    ) -> ApexReturnCode

    @SYS_INITIALIZE_PROCESS_CORE_AFFINITY
    sys_initialize_process_core_affinity(
        process_id: ApexProcessId,
        core_id: ApexProcessorCoreId,
    ) -> ApexReturnCode

    @SYS_GET_MY_PROCESSOR_CORE_ID
    sys_get_my_processor_core_id(
        id: *mut ApexProcessorCoreId,
    ) -> ApexReturnCode

    @SYS_GET_MY_INDEX
    sys_get_my_index(
        index: *mut ApexProcessIndex,
    ) -> ApexReturnCode
}

def_sysfn! {
    @SYS_DEBUG_LOG
    sys_debug_log(
        message: *const u8,
        len: usize,
    ) -> ApexReturnCode

    @SYS_DEBUG_HALT
    sys_debug_halt() -> !
}
