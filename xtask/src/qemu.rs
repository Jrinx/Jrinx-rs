use std::process::{Command, ExitStatus};

use clap::Args;

use crate::{
    arch::ArchArg,
    make::{self, MakeArg},
};

#[derive(Debug, Args, Clone)]
pub struct QemuArg {
    #[clap(long, short = 'M', env = "BOARD", default_value = "virt")]
    pub machine: String,

    #[clap(long, short = 'g')]
    pub gdb: bool,

    #[clap(long, env = "SMP", default_value_t = 1)]
    pub smp: u32,

    #[clap(long, short = 'm', env = "MEMORY", default_value = "1G")]
    pub memory: String,

    #[clap(long, env = "BOOTARGS")]
    pub bootargs: Option<String>,

    #[clap(long, short = 'n')]
    pub no_build: bool,

    #[clap(flatten)]
    pub make_arg: MakeArg,
}

#[must_use]
pub fn run(arg: &QemuArg) -> Option<ExitStatus> {
    let QemuArg {
        machine,
        gdb,
        smp,
        memory,
        bootargs,
        no_build,
        make_arg,
    } = arg.clone();

    if !no_build && !make::run(&make_arg)?.success() {
        return None;
    }

    let MakeArg { arch, .. } = make_arg;
    let ArchArg { arch, .. } = arch;

    let cmd = &mut Command::new(format!("qemu-system-{}", arch));
    cmd.arg("-nographic")
        .arg("-no-reboot")
        .args([
            "-kernel",
            format!(
                "target/{}/{}/jrinx.bin",
                arch,
                std::env::var_os("BUILD_MODE").unwrap().to_str().unwrap(),
            )
            .as_str(),
        ])
        .args(["-M", machine.as_str()])
        .args(["-smp", smp.to_string().as_str()])
        .args(["-m", memory.as_str()]);
    if let Some(bootargs) = bootargs {
        cmd.args(["-append", bootargs.as_str()]);
    }
    if gdb {
        cmd.args(["-s", "-S"]);
    }
    cmd.status().ok()
}
