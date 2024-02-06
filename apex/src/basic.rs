use core::fmt::{Debug, Display};

pub const APEX_NAME_MAX_LEN: usize = 32;
pub const APEX_CORE_AFFINITY_NO_PREFERENCE: ApexProcessorCoreId = -1;

pub const APEX_PRIORITY_MIN: ApexInteger = 0;
pub const APEX_PRIORITY_MAX: ApexInteger = 255;

pub const APEX_LOCK_LEVEL_MIN: ApexInteger = 0;
pub const APEX_LOCK_LEVEL_MAX: ApexInteger = 16;

pub type ApexByte = u8;
pub type ApexInteger = i32;
pub type ApexUnsigned = u32;
pub type ApexLongInteger = i64;
pub type ApexMessageSize = ApexUnsigned;
pub type ApexMessageRange = ApexUnsigned;
pub type ApexWaitingRange = ApexInteger;
pub type ApexProcessorCoreId = ApexInteger;
pub type ApexNumCores = ApexUnsigned;
pub type ApexStackSize = ApexUnsigned;
pub type ApexPriority = ApexInteger;
pub type ApexLockLevel = ApexInteger;

#[repr(transparent)]
#[derive(Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ApexName([u8; APEX_NAME_MAX_LEN]);

impl Display for ApexName {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Debug for ApexName {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let len = self
            .0
            .iter()
            .position(|&c| c == 0)
            .unwrap_or(APEX_NAME_MAX_LEN);
        let s = core::str::from_utf8(&self.0[..len]).unwrap_or("<invalid utf8>");
        write!(f, "{}", s)
    }
}

impl<'a> TryFrom<&'a str> for ApexName {
    type Error = &'a str;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        if value.len() > APEX_NAME_MAX_LEN {
            return Err(value);
        }

        let mut name = [0; APEX_NAME_MAX_LEN];
        name[..value.len()].copy_from_slice(value.as_bytes());

        Ok(Self(name))
    }
}

impl From<ApexName> for [u8; APEX_NAME_MAX_LEN] {
    fn from(name: ApexName) -> Self {
        name.0
    }
}

#[repr(transparent)]
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ApexSystemAddress(usize);

impl From<ApexSystemAddress> for usize {
    fn from(addr: ApexSystemAddress) -> Self {
        addr.0
    }
}

impl ApexSystemAddress {
    pub fn of<R>(f: extern "C" fn() -> R) -> Self {
        Self(f as usize)
    }
}

#[repr(u32)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ApexReturnCode {
    #[default]
    NoError = 0,
    NoAction = 1,
    NotAvailable = 2,
    InvalidParam = 3,
    InvalidConfig = 4,
    InvalidMode = 5,
    TimedOut = 6,
}

impl From<usize> for ApexReturnCode {
    fn from(value: usize) -> Self {
        match value {
            0 => Self::NoError,
            1 => Self::NoAction,
            2 => Self::NotAvailable,
            3 => Self::InvalidParam,
            4 => Self::InvalidConfig,
            5 => Self::InvalidMode,
            6 => Self::TimedOut,
            _ => unimplemented!("Unknown return code: {}", value),
        }
    }
}

impl From<ApexReturnCode> for Result<(), ApexReturnCode> {
    fn from(value: ApexReturnCode) -> Self {
        value.as_result(())
    }
}

impl ApexReturnCode {
    pub fn as_result<T>(self, ok: T) -> Result<T, ApexReturnCode> {
        match self {
            Self::NoError => Ok(ok),
            err => Err(err),
        }
    }
}

#[repr(u32)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ApexDeadline {
    #[default]
    Soft = 0,
    Hard = 1,
}

#[repr(u32)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ApexPortDirection {
    #[default]
    Source = 0,
    Destination = 1,
}

impl TryFrom<ApexUnsigned> for ApexPortDirection {
    type Error = ApexUnsigned;

    fn try_from(value: ApexUnsigned) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Source),
            1 => Ok(Self::Destination),
            _ => Err(value),
        }
    }
}

#[repr(u32)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ApexQueueDiscipline {
    #[default]
    Fifo = 0,
    Priority = 1,
}

impl TryFrom<ApexUnsigned> for ApexQueueDiscipline {
    type Error = ApexUnsigned;

    fn try_from(value: ApexUnsigned) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Fifo),
            1 => Ok(Self::Priority),
            _ => Err(value),
        }
    }
}
