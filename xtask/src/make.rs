use std::{
    env, fs,
    process::{Command, ExitStatus},
};

use clap::Args;

use crate::{
    arch::ArchArg,
    envs,
    util::{cargo::Cargo, CmdOptional},
};

#[derive(Debug, Args, Clone)]
pub struct MakeArg {
    #[clap(long, short = 'd')]
    pub debug: bool,

    #[clap(flatten)]
    pub arch: ArchArg,

    #[clap(long, env = "LOGLEVEL")]
    pub log_level: Option<String>,

    #[clap(long, short = 'f')]
    pub feat: Vec<String>,

    #[clap(long)]
    pub no_default_feat: bool,

    #[clap(long, short = 'B')]
    pub always_make: bool,
}

#[must_use]
pub fn run(arg: &MakeArg) -> Option<ExitStatus> {
    envs::setup(arg);

    let MakeArg {
        always_make, arch, ..
    } = arg.clone();

    let ArchArg { arch, .. } = arch;

    if always_make && !Cargo::new("clean").status().ok()?.success() {
        return None;
    }

    let mut cmd = Cargo::new("build");
    kernel(&mut cmd, arg);

    if !cmd.status().ok()?.success() {
        return None;
    }

    Command::new("rust-objcopy")
        .args(["-O", "binary"])
        .arg(format!("--binary-architecture={}", arch))
        .arg(
            env::current_dir()
                .unwrap()
                .join("target")
                .join(arch.to_string())
                .join(env::var_os("BUILD_MODE").unwrap().to_str().unwrap())
                .join("jrinx"),
        )
        .arg(
            env::current_dir()
                .unwrap()
                .join("target")
                .join(arch.to_string())
                .join(env::var_os("BUILD_MODE").unwrap().to_str().unwrap())
                .join("jrinx.bin"),
        )
        .status()
        .ok()
}

pub fn kernel(cmd: &mut Cargo, arg: &MakeArg) {
    let MakeArg {
        arch,
        feat,
        no_default_feat,
        log_level,
        ..
    } = arg.clone();

    let ArchArg { arch, .. } = arch;

    cmd.package("jrinx")
        .target(
            fs::canonicalize(
                env::current_dir()
                    .unwrap()
                    .join("kern")
                    .join("tgt")
                    .join(format!("{arch}.json")),
            )
            .unwrap()
            .to_str()
            .unwrap(),
        )
        .features(feat)
        .unstable("build-std", ["alloc", "core", "compiler_builtins"])
        .unstable("build-std-features", ["compiler-builtins-mem"])
        .optional(no_default_feat, |cargo| cargo.no_default_features())
        .optional(
            !env::var_os("BUILD_MODE").is_some_and(|mode| mode == "debug"),
            |cargo| cargo.release(),
        )
        .optional(log_level.is_some(), |cargo| {
            cargo.env("LOGLEVEL", log_level.unwrap())
        });
}
