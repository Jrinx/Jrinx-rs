use std::{
    env,
    process::{Command, ExitStatus},
};

use clap::Args;
use rand::Rng;

use crate::arch::{Arch, ArchArg};

#[derive(Debug, Args, Clone)]
pub struct MakeArg {
    #[clap(long, short = 'd')]
    pub debug: bool,

    #[clap(flatten)]
    pub arch: ArchArg,

    #[clap(long, short = 'f')]
    pub feat: Vec<String>,

    #[clap(long)]
    pub no_default_feat: bool,

    #[clap(long, short = 'B')]
    pub always_make: bool,
}

static ARCH_DEFAULT_FEATURES: &[(Arch, &[&str])] = &[
    (Arch::RISCV32, &["pt_level_2"]),
    (Arch::RISCV64, &["pt_level_3"]),
];

static DEFAULT_FEATURES: &[&str] = &["colorful"];

#[must_use]
pub fn run(arg: &MakeArg) -> Option<ExitStatus> {
    setup_envs(arg);

    let MakeArg {
        debug,
        arch,
        feat,
        no_default_feat,
        always_make,
    } = arg.clone();

    let ArchArg { arch, .. } = arch;

    let cmd = &mut Command::new(env!("CARGO"));

    if always_make {
        Command::new(env!("CARGO"))
            .current_dir("kern")
            .arg("clean")
            .status()
            .ok()?;
    }

    cmd.current_dir("kern")
        .arg("build")
        .args(["--target", format!("tgt/{}.json", arch).as_str()])
        .args([
            "--features",
            feat.iter()
                .map(|f| f.as_str())
                .chain(
                    ARCH_DEFAULT_FEATURES
                        .iter()
                        .find(|&&(a, &_)| a == arch)
                        .unwrap()
                        .1
                        .iter()
                        .map(|s| *s),
                )
                .chain(
                    if no_default_feat {
                        [].iter()
                    } else {
                        DEFAULT_FEATURES.iter()
                    }
                    .map(|s| *s),
                )
                .collect::<Vec<_>>()
                .join(",")
                .as_str(),
        ]);
    if !debug {
        cmd.arg("--release");
    }
    cmd.status().ok()
}

fn setup_envs(arg: &MakeArg) {
    let &MakeArg { debug, arch, .. } = arg;
    let ArchArg { arch, .. } = arch;

    let default_envs = [
        ("ARCH", arch.to_string()),
        (
            "BUILD_MODE",
            if debug { "debug" } else { "release" }.to_string(),
        ),
        (
            "BUILD_TIME",
            chrono::offset::Local::now()
                .format("+%Y-%m-%d %H:%M:%S")
                .to_string(),
        ),
        (
            "RAND_SEED",
            rand::thread_rng().gen_range(0..0x8000).to_string(),
        ),
    ];

    for (env_key, env_val) in default_envs {
        if std::env::vars_os().all(|(k, _)| k != env_key) {
            std::env::set_var(env_key, env_val);
        }
    }
}
