use jrinx_a653::partition::Partition;
use jrinx_apex::*;
use jrinx_multitask::runtime::Runtime;

pub(crate) struct PartitionSyscallHandler;

impl PartitionSyscallHandler {
    pub(crate) fn get_status(&self) -> Result<ApexPartitionStatus, ApexReturnCode> {
        Ok(Partition::current().unwrap().status())
    }

    pub(crate) fn set_mode(&self, mode: usize) -> Result<(), ApexReturnCode> {
        let partition = Partition::current().unwrap();
        let current_mode = partition.operating_mode();

        let mode: ApexOperatingMode = (mode as u32)
            .try_into()
            .map_err(|_| ApexReturnCode::InvalidParam)?;
        if mode == ApexOperatingMode::Normal && current_mode == ApexOperatingMode::Normal {
            return Err(ApexReturnCode::NoAction);
        }
        if mode == ApexOperatingMode::WarmStart && current_mode == ApexOperatingMode::ColdStart {
            return Err(ApexReturnCode::InvalidMode);
        }

        partition.set_operating_mode(mode);

        match mode {
            ApexOperatingMode::Idle => todo!("shutdown the partition"),
            ApexOperatingMode::WarmStart | ApexOperatingMode::ColdStart => {
                todo!("inhibit process scheduling and switch back to initialization mode")
            }
            ApexOperatingMode::Normal => {
                // TODO:
                //     [ ] TODO
                //     [-] WIP
                //     [x] DONE
                //
                // [-] set to READY all previously started (not delayed) aperiodic processes
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

                partition.run_pre_start_hooks();
                partition.set_lock_level(APEX_LOCK_LEVEL_MIN);
                partition.pt_sync();
                Runtime::switch_yield();
            }
        }

        Ok(())
    }
}
