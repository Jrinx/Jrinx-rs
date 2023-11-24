use rand::Rng;

use crate::{arch::ArchArg, make::MakeArg};

macro_rules! export_env {
    ($env:literal ?= $val:expr) => {
        if std::env::vars_os().all(|(k, _)| k != $env) {
            std::env::set_var($env, $val);
        }
    };
    ($env0:literal ?= $val0:expr, $($env:literal ?= $val:expr,)+) => {
        export_env!($env0 ?= $val0);
        $(
            export_env!($env ?= $val);
        )+
    };
}

pub fn setup(arg: &MakeArg) {
    let &MakeArg { debug, arch, .. } = arg;
    let ArchArg { arch, .. } = arch;

    export_env! {
        "ARCH" ?= arch.to_string(),
        "UPROG_PATH" ?= std::env::current_dir().unwrap().join("uprog.jrz").to_str().unwrap(),
        "BUILD_MODE" ?= if debug { "debug" } else { "release" },
        "BUILD_TIME" ?= chrono::offset::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        "RAND_SEED" ?= rand::thread_rng().gen_range(0..0x8000).to_string(),
    }
}
