use std::process::ExitStatus;

use crate::{
    uprog::{self, UprogArg},
    util::cargo::Cargo,
};

pub fn run(arg: &UprogArg) -> Option<ExitStatus> {
    let mut cmd = Cargo::new("clippy");

    uprog::user(&mut cmd, arg);

    cmd.args(["--", "-Dwarnings"]);
    cmd.status().ok()
}
