pub mod lint;

use std::{
    env, fs,
    process::{Command, ExitStatus},
};

use clap::Args;
use rand::Rng;

use crate::arch::ArchArg;

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

static DEFAULT_FEATURES: &[&str] = &["colorful"];

#[must_use]
pub fn run(arg: &MakeArg) -> Option<ExitStatus> {
    setup_envs(arg);

    let MakeArg {
        always_make, arch, ..
    } = arg.clone();

    let ArchArg { arch, .. } = arch;

    let kern_path = &fs::canonicalize("kern").unwrap();

    let cmd = &mut Command::new(env!("CARGO"));

    if always_make
        && !Command::new(env!("CARGO"))
            .current_dir(kern_path)
            .arg("clean")
            .status()
            .ok()?
            .success()
    {
        return None;
    }

    cmd.current_dir(kern_path).arg("build");

    construct_build_cmd(arg, cmd);

    if !cmd.status().ok()?.success() {
        return None;
    }

    Command::new("rust-objcopy")
        .args(["-O", "binary"])
        .arg(format!("--binary-architecture={}", arch))
        .arg(format!(
            "target/{}/{}/jrinx",
            arch,
            std::env::var_os("BUILD_MODE").unwrap().to_str().unwrap()
        ))
        .arg(format!(
            "target/{}/{}/jrinx.bin",
            arch,
            std::env::var_os("BUILD_MODE").unwrap().to_str().unwrap()
        ))
        .status()
        .ok()
}

fn construct_build_cmd(arg: &MakeArg, cmd: &mut Command) {
    let MakeArg {
        arch,
        log_level,
        feat,
        no_default_feat,
        ..
    } = arg.clone();

    let ArchArg { arch, .. } = arch;

    cmd.args(["--target", format!("tgt/{}.json", arch).as_str()])
        .args([
            "--features",
            feat.iter()
                .map(|f| f.as_str())
                .chain(
                    if no_default_feat {
                        [].iter()
                    } else {
                        DEFAULT_FEATURES.iter()
                    }
                    .copied(),
                )
                .collect::<Vec<_>>()
                .join(",")
                .as_str(),
        ]);
    if !std::env::var_os("BUILD_MODE").is_some_and(|mode| mode == "debug") {
        cmd.arg("--release");
    }
    if let Some(level) = log_level {
        cmd.env("LOGLEVEL", level);
    }
}

fn setup_envs(arg: &MakeArg) {
    let &MakeArg { debug, arch, .. } = arg;
    let ArchArg { arch, .. } = arch;

    macro_rules! export_env {
        ($env:literal ?= $val:expr) => {
            if std::env::vars_os().all(|(k, _)| k != $env) {
                std::env::set_var($env, $val);
            }
        };
        ($( $env:literal ?= $val:expr ),* ) => {
            $(
                export_env!($env ?= $val);
            )*
        };
    }

    export_env! {
        "ARCH" ?= arch.to_string(),
        "BUILD_MODE" ?= if debug { "debug" } else { "release" },
        "BUILD_TIME" ?= chrono::offset::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        "RAND_SEED" ?= rand::thread_rng().gen_range(0..0x8000).to_string()
    }
}
