use core::ops::Deref;

use alloc::sync::Arc;
use jrinx_a653::{
    partition::Partition,
    process::{Process, ProcessConfig, ProcessRunner},
    A653Entry,
};
use jrinx_apex::*;
use jrinx_hal::{Cpu, Hal, Interrupt};
use jrinx_multitask::runtime::{Runtime, RuntimeStatus};

pub(crate) struct ProcessSyscallHandler;

impl ProcessSyscallHandler {
    pub(crate) fn get_id(&self, name: &ApexProcessName) -> Result<ApexProcessId, ApexReturnCode> {
        let partition = Partition::current().unwrap();
        let process = Process::find_by_name(partition.identifier(), name)
            .ok_or(ApexReturnCode::InvalidConfig)?;

        Ok(process.identifier().into())
    }

    pub(crate) fn get_status(
        &self,
        id: ApexProcessId,
    ) -> Result<ApexProcessStatus, ApexReturnCode> {
        let partition = Partition::current().unwrap();
        let process = Process::find_by_id(partition.identifier(), id.into())
            .ok_or(ApexReturnCode::InvalidParam)?;

        Ok(process.status())
    }

    pub(crate) fn create(
        &self,
        attr: &ApexProcessAttribute,
    ) -> Result<ApexProcessId, ApexReturnCode> {
        let partition = Partition::current().unwrap();
        if Process::find_by_name(partition.identifier(), &attr.name).is_some() {
            return Err(ApexReturnCode::NoAction);
        }
        if attr.stack_size as usize > partition.memory_free() {
            return Err(ApexReturnCode::InvalidParam);
        }
        if attr.base_priority > Process::MAX_PRIORITY {
            return Err(ApexReturnCode::InvalidParam);
        }
        if attr.period != APEX_TIME_INFINITY && attr.period < 0 {
            return Err(ApexReturnCode::InvalidParam);
        }
        if attr.period != APEX_TIME_INFINITY && attr.period % partition.period() != 0 {
            return Err(ApexReturnCode::InvalidConfig);
        }
        if attr.time_capacity != APEX_TIME_INFINITY && attr.time_capacity < 0 {
            return Err(ApexReturnCode::InvalidParam);
        }
        if attr.period != APEX_TIME_INFINITY && attr.time_capacity > attr.period {
            return Err(ApexReturnCode::InvalidParam);
        }
        if partition.operating_mode() == ApexOperatingMode::Normal {
            return Err(ApexReturnCode::InvalidMode);
        }

        let process = Process::new(
            partition.identifier(),
            &ProcessConfig {
                name: attr.name,
                entry: if partition.kernel() {
                    A653Entry::Kern(attr.entry_point)
                } else {
                    A653Entry::User(attr.entry_point.into())
                },
                priority: attr.base_priority,
                deadline: attr.deadline,
                period: attr.period,
                stack_size: attr.stack_size,
                time_capacity: attr.time_capacity,
            },
        )
        .map_err(|_| ApexReturnCode::InvalidConfig)?;

        Ok(process.identifier().into())
    }

    pub(crate) fn start(&self, id: ApexProcessId) -> Result<(), ApexReturnCode> {
        let partition = Partition::current().unwrap();
        let process = Process::find_by_id(partition.identifier(), id.into())
            .ok_or(ApexReturnCode::InvalidParam)?;
        let executor = process
            .gen_executor(ProcessRunner {
                syscall: crate::handle,
            })
            .unwrap();

        let cpu_id = process.core_affinity().unwrap_or(hal!().cpu().id());
        Runtime::with_spec_cpu(cpu_id, move |rt| {
            rt.with_registry(|reg| {
                for (_, inspector) in reg.iter() {
                    let that_partition: Option<Arc<Partition>> =
                        inspector.ext().deref().downcast_ref().cloned();
                    if let Some(that_partition) = that_partition {
                        if that_partition.identifier() == partition.identifier() {
                            inspector.register(executor).unwrap();
                            break;
                        }
                    }
                }
            })
        })
        .unwrap();
        if Runtime::with_spec_cpu(cpu_id, |rt| rt.status() == RuntimeStatus::Endpoint).unwrap() {
            hal!().interrupt().send_ipi(&[cpu_id]);
        }
        Ok(())
    }

    pub(crate) fn initialize_process_core_affinity(
        &self,
        process_id: ApexProcessId,
        core_id: ApexProcessorCoreId,
    ) -> Result<(), ApexReturnCode> {
        let partition = Partition::current().unwrap();
        let process = Process::find_by_id(partition.identifier(), process_id.into())
            .ok_or(ApexReturnCode::InvalidParam)?;

        if !partition.assigned_cores().contains(&core_id) {
            return Err(ApexReturnCode::InvalidConfig);
        }

        if partition.operating_mode() == ApexOperatingMode::Normal {
            return Err(ApexReturnCode::InvalidMode);
        }

        process.set_core_affinity(Some(core_id as _));

        Ok(())
    }
}
