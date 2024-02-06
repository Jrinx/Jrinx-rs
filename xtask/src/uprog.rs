use std::{
    fs,
    path::Path,
    process::{Command, ExitStatus},
};

use cargo_metadata::MetadataCommand;
use clap::Args;

use crate::{
    arch::ArchArg,
    util::{cargo::Cargo, CmdOptional},
};

#[derive(Debug, Args, Clone)]
pub struct UprogArg {
    #[clap(
        long,
        short = 'p',
        default_value_t =
            std::env::current_dir().unwrap()
                .join("user")
                .to_str()
                .unwrap()
                .to_string()
    )]
    pub path: String,

    #[clap(
        long,
        short = 'D',
        default_value_t =
            std::env::current_dir().unwrap()
                .join("uprog")
                .to_str()
                .unwrap()
                .to_string()
    )]
    pub dest: String,

    #[clap(long, short = 'd')]
    pub debug: bool,

    #[clap(long, short = 'k')]
    pub keep_symbols: bool,

    #[clap(flatten)]
    pub arch: ArchArg,

    #[clap(long, short = 'B')]
    pub always_make: bool,
}

#[must_use]
pub fn run(arg: &UprogArg) -> Option<ExitStatus> {
    let UprogArg {
        ref path,
        ref dest,
        debug,
        keep_symbols,
        arch,
        always_make,
        ..
    } = arg.clone();

    let ArchArg { arch, .. } = arch;

    if always_make {
        if !Cargo::new("clean").work_dir(path).status().ok()?.success() {
            return None;
        }

        if Path::exists(Path::new(dest)) && fs::remove_dir_all(dest).is_err() {
            return None;
        }
    }

    let mut cmd = Cargo::new("build");
    user(&mut cmd, arg);

    if !cmd.status().ok()?.success() {
        return None;
    }

    let dest_dir =
        Path::new(dest)
            .join(arch.to_string())
            .join(if debug { "debug" } else { "release" });

    fs::create_dir_all(&dest_dir).unwrap();

    let uprog_meta = MetadataCommand::new()
        .manifest_path(Path::new(path).join("Cargo.toml"))
        .exec()
        .unwrap();

    for prog in uprog_meta.workspace_packages().iter().filter(|pkg| {
        pkg.manifest_path
            .parent()
            .unwrap()
            .canonicalize()
            .unwrap()
            .strip_prefix(Path::new(path).join("programs").canonicalize().unwrap())
            .is_ok()
    }) {
        let bin_file = Path::new(path)
            .join("target")
            .join(arch.triple())
            .join(if debug { "debug" } else { "release" })
            .join(&prog.name);
        let slug = dest_dir.join(
            prog.manifest_path
                .parent()
                .unwrap()
                .canonicalize()
                .unwrap()
                .strip_prefix(Path::new(path).join("programs").canonicalize().unwrap())
                .unwrap(),
        );
        if !keep_symbols
            && !Command::new("rust-strip")
                .arg(bin_file.to_str().unwrap())
                .status()
                .ok()
                .unwrap()
                .success()
        {
            return None;
        }
        fs::create_dir_all(slug.parent().unwrap()).unwrap();
        fs::copy(bin_file, slug).unwrap();
    }

    Some(ExitStatus::default())
}

pub fn user(cmd: &mut Cargo, arg: &UprogArg) {
    let UprogArg {
        path, arch, debug, ..
    } = arg.clone();

    let ArchArg { arch, .. } = arch;

    cmd.work_dir(path)
        .target(arch.triple())
        .optional(!debug, |cargo| cargo.release());
}
