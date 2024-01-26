use core::time::Duration;

use a653rs::bindings::{ApexName, ApexSystemTime, INFINITE_TIME_VALUE, MAX_NAME_LENGTH};
use jrinx_error::InternalError;

pub fn convert_name_to_str(name: &ApexName) -> Result<&str, core::str::Utf8Error> {
    core::str::from_utf8(&name[0..name.iter().position(|&c| c == 0).unwrap_or(name.len())])
}

pub fn convert_str_to_name(s: &str) -> Result<ApexName, InternalError> {
    if s.len() >= MAX_NAME_LENGTH {
        return Err(InternalError::InvalidApexName);
    }
    let mut array_name = [0; MAX_NAME_LENGTH];
    array_name[..s.len()].copy_from_slice(s.as_bytes());
    Ok(array_name)
}

pub fn convert_time_to_duration(time: ApexSystemTime) -> Duration {
    if time == INFINITE_TIME_VALUE {
        Duration::MAX
    } else {
        Duration::from_nanos(time as u64)
    }
}
