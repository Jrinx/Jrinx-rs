use crate::bindings::*;

pub const APEX_TIME_INFINITY: ApexSystemTime = -1;

pub type ApexSystemTime = ApexLongInteger;

pub fn time_as_duration(time: ApexSystemTime) -> core::time::Duration {
    if time == APEX_TIME_INFINITY {
        core::time::Duration::MAX
    } else {
        core::time::Duration::from_nanos(time as u64)
    }
}

pub fn duration_as_time(duration: core::time::Duration) -> ApexSystemTime {
    if duration == core::time::Duration::MAX {
        APEX_TIME_INFINITY
    } else {
        duration.as_nanos() as ApexSystemTime
    }
}
