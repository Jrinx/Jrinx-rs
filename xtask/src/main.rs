mod arch;
mod error;
mod make;
mod qemu;

use std::process::ExitCode;

use clap::{Parser, Subcommand};

use make::MakeArg;
use qemu::QemuArg;

#[derive(Parser)]
#[clap(
    name = "xtask",
    about = "A task runner for building, running and testing Jrinx-rs",
    long_about = None,
)]
struct Cli {
    #[clap(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    MAKE(MakeArg),
    QEMU(QemuArg),
}

fn main() -> ExitCode {
    if let Some(code) = match Cli::parse().cmd {
        Cmd::MAKE(ref arg) => make::run(arg),
        Cmd::QEMU(ref arg) => qemu::run(arg),
    } {
        if code.success() {
            return ExitCode::SUCCESS;
        }
    }
    ExitCode::FAILURE
}
