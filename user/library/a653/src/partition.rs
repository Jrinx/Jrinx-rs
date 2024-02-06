use jrinx_abi::sysfn::*;
use jrinx_apex::*;

pub struct Partition;

impl ApexPartitionService for Partition {
    fn get_partition_status(&self) -> Result<ApexPartitionStatus, ApexReturnCode> {
        let mut status = ApexPartitionStatus::default();
        let code = sys_get_partition_status(&mut status);
        code.as_result(status)
    }

    fn set_partition_mode(&self, operating_mode: ApexOperatingMode) -> Result<(), ApexReturnCode> {
        sys_set_partition_mode(operating_mode).into()
    }
}
