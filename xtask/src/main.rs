mod arch;
mod envs;
mod lint;
mod make;
mod qemu;
mod util;

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
    Make(MakeArg),
    Lint(MakeArg),
    Qemu(QemuArg),
}

fn main() -> ExitCode {
    if let Some(code) = match Cli::parse().cmd {
        Cmd::Make(ref arg) => make::run(arg),
        Cmd::Lint(ref arg) => lint::run(arg),
        Cmd::Qemu(ref arg) => qemu::run(arg),
    } {
        if code.success() {
            return ExitCode::SUCCESS;
        }
    }
    ExitCode::FAILURE
}
