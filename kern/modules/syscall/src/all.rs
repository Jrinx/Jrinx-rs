use alloc::{borrow::ToOwned, boxed::Box, format, string::String};
use core::{future::Future, pin::Pin};

use jrinx_a653::{
    bindings::*, helper::convert_name_to_str, partition::Partition, process::Process,
};
use jrinx_abi::sysno::*;
use jrinx_error::{InternalError, Result};
use jrinx_hal::{Hal, HaltReason};

use crate::partition::PartitionSyscallHandler;
use crate::process::ProcessSyscallHandler;

pub async fn handle(sysno: usize, args: [usize; 7]) -> Result<usize> {
    let ret: core::result::Result<(), ErrorReturnCode> = match sysno {
        SYS_GET_PARTITION_STATUS => {
            let result: &mut ApexPartitionStatus = uptr_try_cast(args[0])?;
            PartitionSyscallHandler
                .get_status()
                .map(|status| *result = status)
        }
        SYS_SET_PARTITION_MODE => PartitionSyscallHandler.set_mode(args[0]),
        SYS_GET_PROCESS_ID => {
            let name: &ProcessName = uptr_try_cast(args[0])?;
            let result: &mut ProcessId = uptr_try_cast(args[1])?;
            ProcessSyscallHandler.get_id(name).map(|id| *result = id)
        }
        SYS_GET_PROCESS_STATUS => {
            let id: ProcessId = args[0] as _;
            let result: &mut ApexProcessStatus = uptr_try_cast(args[1])?;
            ProcessSyscallHandler
                .get_status(id)
                .map(|status| *result = status)
        }
        SYS_CREATE_PROCESS => {
            let attr: &ApexProcessAttribute = uptr_try_cast(args[0])?;
            let result: &mut ProcessId = uptr_try_cast(args[1])?;
            ProcessSyscallHandler.create(attr).map(|id| *result = id)
        }
        SYS_START => ProcessSyscallHandler.start(args[0] as _),
        SYS_INITIALIZE_PROCESS_CORE_AFFINITY => {
            ProcessSyscallHandler.initialize_process_core_affinity(args[0] as _, args[1] as _)
        }
        SYS_DEBUG_LOG => {
            let len: usize = args[1];
            let msg: &[u8] = uptr_try_cast_array(args[0], len)?;
            let partition_name =
                Partition::current().map(|p| convert_name_to_str(&p.name()).unwrap().to_owned());
            let process_name =
                Process::current().map(|p| convert_name_to_str(&p.name()).unwrap().to_owned());
            let prefix = format!(
                "{}|{}",
                partition_name.unwrap_or("<unknown>".to_owned()),
                process_name.unwrap_or("<unknown>".to_owned())
            );
            for line in String::from_utf8_lossy(msg).split('\n') {
                log::debug!("*{}>> {}", prefix, line);
            }
            Ok(())
        }
        SYS_DEBUG_HALT => hal!().halt(HaltReason::NormalExit),
        _ => return Err(InternalError::InvalidSyscallNumber),
    };

    Ok(match ret {
        Ok(()) => 0,
        Err(e) => e as usize,
    })
}

pub fn pinned_handle(
    sysno: usize,
    args: [usize; 7],
) -> Pin<Box<dyn Future<Output = Result<usize>> + Send + Sync>> {
    Box::pin(handle(sysno, args))
}

fn uptr_try_cast<'a, T>(ptr: usize) -> Result<&'a mut T> {
    if ptr >= usize::MAX / 2 || ptr + core::mem::size_of::<T>() > usize::MAX / 2 {
        return Err(InternalError::InvalidVirtAddr);
    }
    if ptr == 0 {
        return Err(InternalError::InvalidVirtAddr);
    }
    Ok(unsafe { &mut *(ptr as *mut T) })
}

fn uptr_try_cast_array<'a, T>(ptr: usize, len: usize) -> Result<&'a mut [T]> {
    if ptr >= usize::MAX / 2 || ptr + len * core::mem::size_of::<T>() > usize::MAX / 2 {
        return Err(InternalError::InvalidVirtAddr);
    }
    if ptr == 0 {
        return Err(InternalError::InvalidVirtAddr);
    }
    Ok(unsafe { core::slice::from_raw_parts_mut(ptr as *mut T, len) })
}
