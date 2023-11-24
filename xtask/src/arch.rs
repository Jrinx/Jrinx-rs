use std::{fmt::Display, str::FromStr};

use clap::Args;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Arch {
    RISCV32,
    RISCV64,
}

impl FromStr for Arch {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "riscv32" => Ok(Self::RISCV32),
            "riscv64" => Ok(Self::RISCV64),
            _ => Err("Unknown architecture".to_string()),
        }
    }
}

impl Display for Arch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Arch::RISCV32 => write!(f, "riscv32"),
            Arch::RISCV64 => write!(f, "riscv64"),
        }
    }
}

impl Arch {
    pub fn triple(&self) -> &str {
        match self {
            Arch::RISCV32 => "riscv32imac-unknown-none-elf",
            Arch::RISCV64 => "riscv64gc-unknown-none-elf",
        }
    }
}

#[derive(Debug, Args, Clone, Copy)]
pub struct ArchArg {
    #[clap(long, short = 'a', env = "ARCH")]
    pub arch: Arch,
}
