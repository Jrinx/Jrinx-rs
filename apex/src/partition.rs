use crate::bindings::*;

pub type ApexPartitionId = ApexLongInteger;

#[repr(u32)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ApexOperatingMode {
    #[default]
    Idle = 0,
    ColdStart = 1,
    WarmStart = 2,
    Normal = 3,
}

impl TryFrom<ApexUnsigned> for ApexOperatingMode {
    type Error = ApexUnsigned;

    fn try_from(value: ApexUnsigned) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Idle),
            1 => Ok(Self::ColdStart),
            2 => Ok(Self::WarmStart),
            3 => Ok(Self::Normal),
            _ => Err(value),
        }
    }
}

#[repr(u32)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ApexStartCondition {
    #[default]
    NormalStart = 0,
    PartitionRestart = 1,
    HmModuleRestart = 2,
    HmPartitionRestart = 3,
}

impl TryFrom<ApexUnsigned> for ApexStartCondition {
    type Error = ApexUnsigned;

    fn try_from(value: ApexUnsigned) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::NormalStart),
            1 => Ok(Self::PartitionRestart),
            2 => Ok(Self::HmModuleRestart),
            3 => Ok(Self::HmPartitionRestart),
            _ => Err(value),
        }
    }
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ApexPartitionStatus {
    pub period: ApexSystemTime,
    pub duration: ApexSystemTime,
    pub identifier: ApexPartitionId,
    pub lock_level: ApexLockLevel,
    pub operating_mode: ApexOperatingMode,
    pub start_condition: ApexStartCondition,
    pub num_assigned_cores: ApexNumCores,
}

pub trait ApexPartitionService {
    fn get_partition_status(&self) -> Result<ApexPartitionStatus, ApexReturnCode>;

    fn set_partition_mode(&self, operating_mode: ApexOperatingMode) -> Result<(), ApexReturnCode>;
}
