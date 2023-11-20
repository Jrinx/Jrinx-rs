use std::{env, process::ExitStatus};

use clap::Args;

use crate::{
    arch::ArchArg,
    make::{self, MakeArg},
    util::{qemu::Qemu, CmdOptional},
};

#[derive(Debug, Args, Clone)]
pub struct QemuArg {
    #[clap(long, short = 'M', env = "BOARD", default_value = "virt")]
    pub machine: String,

    #[clap(long, short = 'g')]
    pub gdb: bool,

    #[clap(long, env = "SMP", default_value_t = 5)]
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

    Qemu::new(&arch.to_string())
        .kernel(
            env::current_dir()
                .unwrap()
                .join("target")
                .join(arch.to_string())
                .join(env::var_os("BUILD_MODE").unwrap().to_str().unwrap())
                .join("jrinx.bin"),
        )
        .machine(&machine)
        .memory(&memory)
        .smp(smp as _)
        .no_graphic()
        .no_reboot()
        .optional(bootargs.is_some(), |qemu| {
            qemu.bootargs(bootargs.unwrap().as_str())
        })
        .optional(gdb, |qemu| qemu.gdb_server())
        .status()
        .ok()
}
