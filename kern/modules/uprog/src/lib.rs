#![no_std]

use elf::{endian::AnyEndian, ElfBytes};
use jrinx_error::{InternalError, Result};

static USER_PROGRAMS: &[u8] = include_bytes!(core::env!("UPROG_PATH"));

pub fn all() -> impl Iterator<Item = &'static str> {
    cpio_reader::iter_files(USER_PROGRAMS).map(|entry| entry.name())
}

pub fn find(slug: &str) -> Result<ElfBytes<'static, AnyEndian>> {
    cpio_reader::iter_files(USER_PROGRAMS)
        .find_map(|entry| {
            let name = entry.name();
            let content = entry.file();
            if name == slug {
                Some(ElfBytes::minimal_parse(content).map_err(|_| InternalError::ElfParseError))
            } else {
                None
            }
        })
        .unwrap_or(Err(InternalError::ElfParseError))
}
