mod ar;
mod arch;
mod envs;
mod lint;
mod make;
mod qemu;
mod uprog;
mod util;

use std::process::ExitCode;

use clap::{Parser, Subcommand};

use ar::ArchiveArg;
use make::MakeArg;
use qemu::QemuArg;
use uprog::UprogArg;

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
    Uprog(UprogArg),
    Ar(ArchiveArg),
}

fn main() -> ExitCode {
    if let Some(code) = match Cli::parse().cmd {
        Cmd::Make(ref arg) => make::run(arg),
        Cmd::Lint(ref arg) => lint::run(arg),
        Cmd::Qemu(ref arg) => qemu::run(arg),
        Cmd::Uprog(ref arg) => uprog::run(arg),
        Cmd::Ar(ref arg) => ar::run(arg),
    } {
        if code.success() {
            return ExitCode::SUCCESS;
        }
    }
    ExitCode::FAILURE
}
