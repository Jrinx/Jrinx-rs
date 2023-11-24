use std::{env, fs, path::Path, process::ExitStatus};

use crate::{
    envs,
    make::{self, MakeArg},
    util::cargo::Cargo,
};

pub fn run(arg: &MakeArg) -> Option<ExitStatus> {
    envs::setup(arg);

    let uprog_path = env::var_os("UPROG_PATH").unwrap();
    let uprog_path = Path::new(uprog_path.to_str().unwrap());

    if !Path::exists(uprog_path) {
        fs::write(uprog_path, "MOCK UPROG BINARY").unwrap();
    }

    let mut cmd = Cargo::new("clippy");

    make::kernel(&mut cmd, arg);

    cmd.args(["--", "-Dwarnings"]);
    cmd.status().ok()
}
