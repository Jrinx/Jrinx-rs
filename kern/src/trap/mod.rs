pub mod breakpoint;
pub mod interrupt;
pub mod timer_int;

use jrinx_addr::VirtAddr;
use jrinx_paging::PagePerm;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrapReason {
    Interrupt(usize),
    SystemCall,
    Breakpoint { addr: VirtAddr },
    PageFault { addr: VirtAddr, perm: PagePerm },
    Unknown { code: usize },
}
