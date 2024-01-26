macro_rules! def_sysno {
    ($name:ident = $value:expr) => {
        pub const $name: usize = $value;
    };

    ($name:ident = $value:expr, $($rest_name:ident = $rest_value:expr,)*) => {
        def_sysno!($name = $value);
        $(
            def_sysno!($rest_name = $rest_value)
        )*
    };

    ($name:ident = $value:expr, $rest:ident) => {
        def_sysno!($name = $value);
        def_sysno!($rest = $value + 1);
    };

    ($name:ident = $value:expr, $next:ident, $($rest:ident,)*) => {
        def_sysno!($name = $value);
        def_sysno!($next = $value + 1, $($rest,)*);
    };
}

def_sysno! {
    SYS_GET_PARTITION_STATUS = 0x1000,
    SYS_SET_PARTITION_MODE,
}

def_sysno! {
    SYS_GET_PROCESS_ID = 0x2000,
    SYS_GET_PROCESS_STATUS,
    SYS_CREATE_PROCESS,
    SYS_SET_PRIORITY,
    SYS_SUSPEND_SELF,
    SYS_SUSPEND,
    SYS_RESUME,
    SYS_STOP_SELF,
    SYS_STOP,
    SYS_START,
    SYS_DELAYED_START,
    SYS_LOCK_PREEMPTION,
    SYS_UNLOCK_PREEMPTION,
    SYS_GET_MY_ID,
    SYS_INITIALIZE_PROCESS_CORE_AFFINITY,
    SYS_GET_MY_PROCESSOR_CORE_ID,
    SYS_GET_MY_INDEX,
}

def_sysno! {
    SYS_TIMED_WAIT = 0x3000,
    SYS_PERIODIC_WAIT,
    SYS_GET_TIME,
    SYS_REPLENISH,
}

def_sysno! {
    SYS_CREATE_SAMPLING_PORT = 0x4000,
    SYS_WRITE_SAMPLING_MESSAGE,
    SYS_READ_SAMPLING_MESSAGE,
    SYS_GET_SAMPLING_PORT_ID,
    SYS_GET_SAMPLING_PORT_STATUS,
}

def_sysno! {
    SYS_CREATE_QUEUING_PORT = 0x4100,
    SYS_SEND_QUEUING_MESSAGE,
    SYS_RECEIVE_QUEUING_MESSAGE,
    SYS_GET_QUEUING_PORT_ID,
    SYS_GET_QUEUING_PORT_STATUS,
    SYS_CLEAR_QUEUING_PORT,
}

def_sysno! {
    SYS_CREATE_BUFFER = 0x5000,
    SYS_SEND_BUFFER,
    SYS_RECEIVE_BUFFER,
    SYS_GET_BUFFER_ID,
    SYS_GET_BUFFER_STATUS,
}

def_sysno! {
    SYS_CREATE_BLACKBOARD = 0x5100,
    SYS_DISPLAY_BLACKBOARD,
    SYS_READ_BLACKBOARD,
    SYS_CLEAR_BLACKBOARD,
    SYS_GET_BLACKBOARD_ID,
    SYS_GET_BLACKBOARD_STATUS,
}

def_sysno! {
    SYS_CREATE_SEMAPHORE = 0x5200,
    SYS_WAIT_SEMAPHORE,
    SYS_SIGNAL_SEMAPHORE,
    SYS_GET_SEMAPHORE_ID,
    SYS_GET_SEMAPHORE_STATUS,
}

def_sysno! {
    SYS_CREATE_EVENT = 0x5300,
    SYS_SET_EVENT,
    SYS_RESET_EVENT,
    SYS_WAIT_EVENT,
    SYS_GET_EVENT_ID,
    SYS_GET_EVENT_STATUS,
}

def_sysno! {
    SYS_CREATE_MUTEX = 0x5400,
    SYS_ACQUIRE_MUTEX,
    SYS_RELEASE_MUTEX,
    SYS_RESET_MUTEX,
    SYS_GET_MUTEX_ID,
    SYS_GET_MUTEX_STATUS,
    SYS_GET_PROCESS_MUTEX_STATE,
}

def_sysno! {
    SYS_DEBUG_LOG = 0xdbdbdbdb,
    SYS_DEBUG_HALT,
}
