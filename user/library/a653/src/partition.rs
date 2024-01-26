use a653rs::bindings::*;
use jrlib_sys::*;

pub struct Partition;

impl ApexPartitionP4 for Partition {
    fn get_partition_status() -> ApexPartitionStatus {
        let mut status = ApexPartitionStatus::default();
        sys_get_partition_status(&mut status);
        status
    }

    fn set_partition_mode(operating_mode: OperatingMode) -> Result<(), ErrorReturnCode> {
        ErrorReturnCode::from(sys_set_partition_mode(operating_mode))
    }
}
