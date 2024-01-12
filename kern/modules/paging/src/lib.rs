#![no_std]
#![feature(allocator_api)]
#![feature(asm_const)]
#![feature(naked_functions)]

extern crate alloc;

mod arch;
pub use arch::*;

pub mod common;

use alloc::sync::Arc;
use core::fmt::Display;

use jrinx_addr::{PhysAddr, VirtAddr};
use jrinx_error::Result;
use jrinx_phys_frame::PhysFrame;

pub trait GenericPagePerm: Clone + Copy + Display + Send + Sync + bitflags::Flags {
    const V: Self;
    const R: Self;
    const W: Self;
    const X: Self;
    const U: Self;
    const G: Self;
}

pub trait GenericPageTableEntry<P: GenericPagePerm>:
    Clone + Into<(PhysAddr, P)> + Into<usize>
{
    fn valid(&self) -> bool {
        let (_, perm): (PhysAddr, P) = self.clone().into();
        perm.contains(P::V)
    }

    fn set(&mut self, phys_addr: PhysAddr, perm: P);

    fn clr(&mut self);
}

pub(crate) trait CloneKernel {
    fn clone_kernel(dst: &mut [usize]);
}

pub trait GenericPageTable<P, E>
where
    P: GenericPagePerm,
    E: GenericPageTableEntry<P>,
{
    fn addr(&self) -> PhysAddr;

    fn translate(&self, addr: VirtAddr) -> Result<(PhysAddr, P)>;

    fn lookup(&self, addr: VirtAddr) -> Result<(Arc<PhysFrame>, P)>;

    fn map(&mut self, addr: VirtAddr, phys_frame: Arc<PhysFrame>, perm: P) -> Result<()>;

    fn unmap(&mut self, addr: VirtAddr) -> Result<()>;
}
