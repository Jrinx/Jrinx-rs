use std::{
    fs,
    process::{Command, ExitStatus},
};

use super::{construct_build_cmd, setup_envs, MakeArg};

pub fn run(arg: &MakeArg) -> Option<ExitStatus> {
    setup_envs(arg);

    let cmd = &mut Command::new(env!("CARGO"));

    let kern_path = &fs::canonicalize("kern").unwrap();

    cmd.current_dir(kern_path).arg("clippy");

    construct_build_cmd(arg, cmd);

    cmd.args(["--", "-Dwarnings"]);
    cmd.status().ok()
}
