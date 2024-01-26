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
        F: FnMut(&ElfBytes<'_, AnyEndian>, &ProgramHeader, VirtAddr, usize, usize) -> Result<()>,
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
        F: FnMut(&ElfBytes<'_, AnyEndian>, &ProgramHeader, VirtAddr, usize, usize) -> Result<()>,
    {
        let vaddr = VirtAddr::new(seg_header.p_vaddr as usize);
        let fsize = seg_header.p_filesz as usize;
        let msize = seg_header.p_memsz as usize;

        let head_part_len = vaddr - vaddr.align_page_down();

        if head_part_len != 0 {
            loader(
                self.elf,
                seg_header,
                vaddr,
                0,
                cmp::min(fsize, PAGE_SIZE - head_part_len),
            )?;
        }

        for i in (if head_part_len != 0 {
            cmp::min(fsize, PAGE_SIZE - head_part_len)
        } else {
            0
        }..fsize)
            .step_by(PAGE_SIZE)
        {
            loader(
                self.elf,
                seg_header,
                vaddr + i,
                i,
                cmp::min(fsize - i, PAGE_SIZE),
            )?;
        }

        for i in (fsize..msize).step_by(PAGE_SIZE) {
            loader(self.elf, seg_header, vaddr + i, i, 0)?;
        }

        Ok(())
    }
}
