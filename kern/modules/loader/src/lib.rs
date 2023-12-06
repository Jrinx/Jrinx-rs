#![no_std]

use core::cmp;

use elf::{abi::PT_LOAD, endian::AnyEndian, segment::ProgramHeader, ElfBytes};
use jrinx_addr::VirtAddr;
use jrinx_config::PAGE_SIZE;
use jrinx_error::{InternalError, Result};

pub struct ElfLoader<'elf, 'a> {
    elf: &'elf ElfBytes<'a, AnyEndian>,
}

impl<'elf, 'a> ElfLoader<'elf, 'a> {
    pub fn new(elf: &'elf ElfBytes<'a, AnyEndian>) -> Self {
        Self { elf }
    }

    pub fn load<F>(&self, mut loader: F) -> Result<()>
    where
        F: FnMut(&ElfBytes<'a, AnyEndian>, &ProgramHeader, VirtAddr, usize, usize) -> Result<()>,
    {
        for seg_header in self
            .elf
            .segments()
            .ok_or(InternalError::ElfParseError)?
            .iter()
            .filter(|seg_header| seg_header.p_type == PT_LOAD)
        {
            self.load_segment(&mut loader, &seg_header)?;
        }

        Ok(())
    }

    fn load_segment<F>(&self, mut loader: F, seg_header: &ProgramHeader) -> Result<()>
    where
        F: FnMut(&ElfBytes<'a, AnyEndian>, &ProgramHeader, VirtAddr, usize, usize) -> Result<()>,
    {
        let seg_vaddr = VirtAddr::new(seg_header.p_vaddr as usize);
        let offset = seg_vaddr - seg_vaddr.align_page_down();
        let offset_len =
            (offset != 0).then_some(cmp::min(seg_header.p_filesz as usize, PAGE_SIZE - offset));

        let region_to_load = offset_len.unwrap_or(0)..seg_header.p_filesz as usize;
        let region_to_zero = seg_header.p_filesz as usize..seg_header.p_memsz as usize;

        if let Some(len) = offset_len {
            loader(self.elf, seg_header, seg_vaddr, offset, len)?;
        }

        for vaddr in region_to_load
            .step_by(PAGE_SIZE)
            .map(|offset| seg_vaddr + offset)
        {
            loader(
                self.elf,
                seg_header,
                vaddr,
                0,
                cmp::min(
                    PAGE_SIZE,
                    seg_header.p_filesz as usize - (vaddr - seg_vaddr),
                ),
            )?;
        }

        for vaddr in region_to_zero
            .step_by(PAGE_SIZE)
            .map(|offset| seg_vaddr + offset)
        {
            loader(self.elf, seg_header, vaddr, 0, 0)?;
        }

        Ok(())
    }
}
