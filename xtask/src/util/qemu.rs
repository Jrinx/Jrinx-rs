use std::{
    ffi::OsStr,
    ops::{Deref, DerefMut},
    process::Command,
};

use super::CmdOptional;

pub struct Qemu {
    cmd: Command,
}

impl Qemu {
    pub fn new(arch: &str) -> Self {
        Self {
            cmd: Command::new(format!("qemu-system-{arch}")),
        }
    }
    pub fn kernel<S: AsRef<OsStr>>(&mut self, path: S) -> &mut Self {
        self.args(["-kernel", path.as_ref().to_str().unwrap()]);
        self
    }

    pub fn machine(&mut self, machine: &str) -> &mut Self {
        self.args(["-M", machine]);
        self
    }

    pub fn smp(&mut self, num: usize) -> &mut Self {
        self.args(["-smp", num.to_string().as_str()]);
        self
    }

    pub fn memory(&mut self, size: &str) -> &mut Self {
        self.args(["-m", size]);
        self
    }

    pub fn bootargs(&mut self, bootargs: &str) -> &mut Self {
        self.args(["-append", bootargs]);
        self
    }

    pub fn gdb_server(&mut self) -> &mut Self {
        self.args(["-s", "-S"]);
        self
    }

    pub fn no_graphic(&mut self) -> &mut Self {
        self.arg("-nographic");
        self
    }

    pub fn no_reboot(&mut self) -> &mut Self {
        self.arg("-no-reboot");
        self
    }
}

impl CmdOptional for Qemu {}

impl Deref for Qemu {
    type Target = Command;

    fn deref(&self) -> &Self::Target {
        &self.cmd
    }
}

impl DerefMut for Qemu {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.cmd
    }
}
