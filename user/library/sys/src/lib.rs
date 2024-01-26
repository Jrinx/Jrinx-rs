#![no_std]

mod macros;

use core::arch::asm;

use a653rs::bindings::*;
use jrinx_abi::sysno::*;

use crate::macros::*;

def_sys_fn! {
    @SYS_GET_PARTITION_STATUS
    sys_get_partition_status(
        status: *mut ApexPartitionStatus,
    ) -> ReturnCode

    @SYS_SET_PARTITION_MODE
    sys_set_partition_mode(
        mode: OperatingMode,
    ) -> ReturnCode
}

def_sys_fn! {
    @SYS_GET_PROCESS_ID
    sys_get_process_id(
        name: *const ProcessName,
        id: *mut ProcessId,
    ) -> ReturnCode

    @SYS_GET_PROCESS_STATUS
    sys_get_process_status(
        id: ProcessId,
        status: *mut ApexProcessStatus,
    ) -> ReturnCode

    @SYS_CREATE_PROCESS
    sys_create_process(
        attr: *const ApexProcessAttribute,
        id: *mut ProcessId,
    ) -> ReturnCode

    @SYS_SET_PRIORITY
    sys_set_priority(
        id: ProcessId,
        priority: Priority,
    ) -> ReturnCode

    @SYS_SUSPEND_SELF
    sys_suspend_self(
        timeout: ApexSystemTime,
    ) -> ReturnCode

    @SYS_SUSPEND
    sys_suspend(
        id: ProcessId,
    ) -> ReturnCode

    @SYS_RESUME
    sys_resume(
        id: ProcessId,
    ) -> ReturnCode

    @SYS_STOP_SELF
    sys_stop_self() -> !

    @SYS_STOP
    sys_stop(
        id: ProcessId,
    ) -> ReturnCode

    @SYS_START
    sys_start(
        id: ProcessId,
    ) -> ReturnCode

    @SYS_DELAYED_START
    sys_delayed_start(
        id: ProcessId,
        delay_time: ApexSystemTime,
    ) -> ReturnCode

    @SYS_LOCK_PREEMPTION
    sys_lock_preemption(
        lock_level: LockLevel,
    ) -> ReturnCode

    @SYS_UNLOCK_PREEMPTION
    sys_unlock_preemption(
        lock_level: *mut LockLevel,
    ) -> ReturnCode

    @SYS_GET_MY_ID
    sys_get_my_id(
        id: *mut ProcessId,
    ) -> ReturnCode

    @SYS_INITIALIZE_PROCESS_CORE_AFFINITY
    sys_initialize_process_core_affinity(
        process_id: ProcessId,
        core_id: ProcessorCoreId,
    ) -> ReturnCode

    @SYS_GET_MY_PROCESSOR_CORE_ID
    sys_get_my_processor_core_id(
        id: *mut ProcessorCoreId,
    ) -> ReturnCode

    @SYS_GET_MY_INDEX
    sys_get_my_index(
        index: *mut ProcessIndex,
    ) -> ReturnCode
}

def_sys_fn! {
    @SYS_TIMED_WAIT
    sys_timed_wait(
        delay_time: ApexSystemTime,
    ) -> ReturnCode

    @SYS_PERIODIC_WAIT
    sys_periodic_wait() -> ReturnCode

    @SYS_GET_TIME
    sys_get_time(
        time: *mut ApexSystemTime,
    ) -> ReturnCode

    @SYS_REPLENISH
    sys_replenish(
        budget: ApexSystemTime,
    ) -> ReturnCode
}

def_sys_fn! {
    @SYS_CREATE_SAMPLING_PORT
    sys_create_sampling_port(
        name: *const SamplingPortName,
        max_message_size: MessageSize,
        port_direction: PortDirection,
        refresh_period: ApexSystemTime,
        port_id: *mut SamplingPortId,
    ) -> ReturnCode

    @SYS_WRITE_SAMPLING_MESSAGE
    sys_write_sampling_message(
        port_id: SamplingPortId,
        message: *const ApexByte,
        len: MessageSize,
    ) -> ReturnCode

    @SYS_READ_SAMPLING_MESSAGE
    sys_read_sampling_message(
        port_id: SamplingPortId,
        message: *mut ApexByte,
        len: *mut MessageSize,
        validity: *mut Validity,
    ) -> ReturnCode

    @SYS_GET_SAMPLING_PORT_ID
    sys_get_sampling_port_id(
        name: *const SamplingPortName,
        port_id: *mut SamplingPortId,
    ) -> ReturnCode

    // FIXME: @SYS_GET_SAMPLING_PORT_STATUS repr(C)

    @SYS_CREATE_QUEUING_PORT
    sys_create_queuing_port(
        name: *const QueuingPortName,
        max_message_size: MessageSize,
        max_nb_messages: MessageRange,
        port_direction: PortDirection,
        queuing_discipline: QueuingDiscipline,
        port_id: *mut QueuingPortId,
    ) -> ReturnCode

    @SYS_SEND_QUEUING_MESSAGE
    sys_send_queuing_message(
        port_id: QueuingPortId,
        message: *const ApexByte,
        len: MessageSize,
        time_out: ApexSystemTime,
    ) -> ReturnCode

    @SYS_RECEIVE_QUEUING_MESSAGE
    sys_receive_queuing_message(
        port_id: QueuingPortId,
        time_out: ApexSystemTime,
        message: *mut ApexByte,
        len: *mut MessageSize,
    ) -> ReturnCode

    @SYS_GET_QUEUING_PORT_ID
    sys_get_queuing_port_id(
        name: *const QueuingPortName,
        port_id: *mut QueuingPortId,
    ) -> ReturnCode

    // FIXME: @SYS_GET_QUEUING_PORT_STATUS repr(C)

    @SYS_CLEAR_QUEUING_PORT
    sys_clear_queuing_port(
        port_id: QueuingPortId,
    ) -> ReturnCode
}

def_sys_fn! {
    @SYS_CREATE_BUFFER
    sys_create_buffer(
        name: *const BufferName,
        max_message_size: MessageSize,
        max_nb_messages: MessageRange,
        queuing_discipline: QueuingDiscipline,
        buffer_id: *mut BufferId,
    ) -> ReturnCode

    @SYS_SEND_BUFFER
    sys_send_buffer(
        buffer_id: BufferId,
        message: *const ApexByte,
        len: MessageSize,
        time_out: ApexSystemTime,
    ) -> ReturnCode

    @SYS_RECEIVE_BUFFER
    sys_receive_buffer(
        buffer_id: BufferId,
        time_out: ApexSystemTime,
        message: *mut ApexByte,
        len: *mut MessageSize,
    ) -> ReturnCode

    @SYS_GET_BUFFER_ID
    sys_get_buffer_id(
        name: *const BufferName,
        buffer_id: *mut BufferId,
    ) -> ReturnCode

    // FIXME: @SYS_GET_BUFFER_STATUS repr(C)

    @SYS_CREATE_BLACKBOARD
    sys_create_blackboard(
        name: *const BlackboardName,
        max_message_size: MessageSize,
        blackboard_id: *mut BlackboardId,
    ) -> ReturnCode

    @SYS_DISPLAY_BLACKBOARD
    sys_display_blackboard(
        blackboard_id: BlackboardId,
        message: *const ApexByte,
        len: MessageSize,
    ) -> ReturnCode

    @SYS_READ_BLACKBOARD
    sys_read_blackboard(
        blackboard_id: BlackboardId,
        time_out: ApexSystemTime,
        message: *mut ApexByte,
        len: *mut MessageSize,
    ) -> ReturnCode

    @SYS_CLEAR_BLACKBOARD
    sys_clear_blackboard(
        blackboard_id: BlackboardId,
    ) -> ReturnCode

    @SYS_GET_BLACKBOARD_ID
    sys_get_blackboard_id(
        name: *const BlackboardName,
        blackboard_id: *mut BlackboardId,
    ) -> ReturnCode

    // FIXME: @SYS_GET_BLACKBOARD_STATUS repr(C)

    @SYS_CREATE_SEMAPHORE
    sys_create_semaphore(
        name: *const SemaphoreName,
        current_value: SemaphoreValue,
        maximum_value: SemaphoreValue,
        queuing_discipline: QueuingDiscipline,
        semaphore_id: *mut SemaphoreId,
    ) -> ReturnCode

    @SYS_WAIT_SEMAPHORE
    sys_wait_semaphore(
        semaphore_id: SemaphoreId,
        time_out: ApexSystemTime,
    ) -> ReturnCode

    @SYS_SIGNAL_SEMAPHORE
    sys_signal_semaphore(
        semaphore_id: SemaphoreId,
    ) -> ReturnCode

    @SYS_GET_SEMAPHORE_ID
    sys_get_semaphore_id(
        name: *const SemaphoreName,
        semaphore_id: *mut SemaphoreId,
    ) -> ReturnCode

    @SYS_GET_SEMAPHORE_STATUS
    sys_get_semaphore_status(
        semaphore_id: SemaphoreId,
        status: *mut SemaphoreStatus,
    ) -> ReturnCode

    @SYS_CREATE_EVENT
    sys_create_event(
        name: *const EventName,
        event_id: *mut EventId,
    ) -> ReturnCode

    @SYS_SET_EVENT
    sys_set_event(
        event_id: EventId,
    ) -> ReturnCode

    @SYS_RESET_EVENT
    sys_reset_event(
        event_id: EventId,
    ) -> ReturnCode

    @SYS_WAIT_EVENT
    sys_wait_event(
        event_id: EventId,
        time_out: ApexSystemTime,
    ) -> ReturnCode

    @SYS_GET_EVENT_ID
    sys_get_event_id(
        name: *const EventName,
        event_id: *mut EventId,
    ) -> ReturnCode

    // FIXME: @SYS_GET_EVENT_STATUS repr(C)

    @SYS_CREATE_MUTEX
    sys_create_mutex(
        name: *const MutexName,
        priority: Priority,
        queuing_discipline: QueuingDiscipline,
        mutex_id: *mut MutexId,
    ) -> ReturnCode

    @SYS_ACQUIRE_MUTEX
    sys_acquire_mutex(
        mutex_id: MutexId,
        time_out: ApexSystemTime,
    ) -> ReturnCode

    @SYS_RELEASE_MUTEX
    sys_release_mutex(
        mutex_id: MutexId,
    ) -> ReturnCode

    @SYS_RESET_MUTEX
    sys_reset_mutex(
        mutex_id: MutexId,
        process_id: ProcessId,
    ) -> ReturnCode

    @SYS_GET_MUTEX_ID
    sys_get_mutex_id(
        name: *const MutexName,
        mutex_id: *mut MutexId,
    ) -> ReturnCode

    // TODO: @SYS_GET_MUTEX_STATUS repr(C)

    @SYS_GET_PROCESS_MUTEX_STATE
    sys_get_process_mutex_state(
        process_id: ProcessId,
        mutex_id: *mut MutexId,
    ) -> ReturnCode
}

def_sys_fn! {
    @SYS_DEBUG_LOG
    sys_debug_log(
        message: *const u8,
        len: usize,
    ) -> ReturnCode

    @SYS_DEBUG_HALT
    sys_debug_halt() -> !
}
