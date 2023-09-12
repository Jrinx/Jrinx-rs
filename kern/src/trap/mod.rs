use crate::{arch::mm::virt::PagePerm, mm::virt::VirtAddr};

pub mod breakpoint;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrapReason {
    Interrupt(usize),
    Breakpoint { addr: VirtAddr },
    PageFault { addr: VirtAddr, perm: PagePerm },
    Unknown,
}
