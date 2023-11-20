use std::process::ExitStatus;

use crate::{
    envs,
    make::{self, MakeArg},
    util::cargo::Cargo,
};

pub fn run(arg: &MakeArg) -> Option<ExitStatus> {
    envs::setup(arg);

    let mut cmd = Cargo::new("clippy");

    make::kernel(&mut cmd, arg);

    cmd.args(["--", "-Dwarnings"]);
    cmd.status().ok()
}
