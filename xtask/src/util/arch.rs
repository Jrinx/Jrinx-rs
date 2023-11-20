use std::{fmt::Display, str::FromStr};

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
