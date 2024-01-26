use jrinx_a653::{
    bindings::{ApexPartitionStatus, ErrorReturnCode, OperatingMode, MIN_LOCK_LEVEL},
    partition::Partition,
};
use jrinx_multitask::runtime::Runtime;

pub(crate) struct PartitionSyscallHandler;

impl PartitionSyscallHandler {
    pub(crate) async fn get_status(&self) -> Result<ApexPartitionStatus, ErrorReturnCode> {
        Ok(Partition::current().unwrap().status())
    }

    pub(crate) async fn set_mode(&self, mode: usize) -> Result<(), ErrorReturnCode> {
        let partition = Partition::current().unwrap();
        let current_mode = partition.operating_mode();

        let mode: OperatingMode = (mode as u32)
            .try_into()
            .map_err(|_| ErrorReturnCode::InvalidParam)?;
        if mode == OperatingMode::Normal && current_mode == OperatingMode::Normal {
            return Err(ErrorReturnCode::NoAction);
        }
        if mode == OperatingMode::WarmStart && current_mode == OperatingMode::ColdStart {
            return Err(ErrorReturnCode::InvalidMode);
        }

        partition.set_operating_mode(mode);

        match mode {
            OperatingMode::Idle => todo!("shutdown the partition"),
            OperatingMode::WarmStart | OperatingMode::ColdStart => {
                todo!("inhibit process scheduling and switch back to initialization mode")
            }
            OperatingMode::Normal => {
                // TODO:
                // [ ] set to READY all previously started (not delayed) aperiodic processes
                //     (unless the process was suspended);
                // [ ] set release point of all previously delay started aperiodic processes
                //     to the system clock time plus their delay times;
                // [ ] set first release points of all previously started (not delayed) periodic
                //     processes to the partition’s next periodic processing start;
                // [ ]  set first release points of all previously delay started periodic processes
                //     to the partition’s next periodic processing start plus their delay times;
                //     -- at their release points, the processes are set to READY (if not DORMANT)
                // [ ] calculate the DEADLINE_TIME of all non-dormant processes in the partition;
                //     -- a DEADLINE_TIME calculation may cause an overflow of the underlying
                //     -- clock. If this occurs, HM is invoked with an illegal request error code
                // [x] set the partition’s lock level to zero;
                // [ ] if (an error handler process has been created) then
                //       enable the error handler process for execution and fault processing;
                //     end if;
                // [x] activate the process scheduling;

                partition.set_lock_level(MIN_LOCK_LEVEL);
                partition.pt_sync();
                Runtime::switch_yield();
            }
        }

        Ok(())
    }
}
