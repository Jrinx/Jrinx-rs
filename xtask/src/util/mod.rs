pub mod arch;
pub mod cargo;
pub mod qemu;

pub trait CmdOptional {
    fn optional(&mut self, pred: bool, f: impl FnOnce(&mut Self) -> &mut Self) -> &mut Self {
        if pred {
            f(self);
        }
        self
    }
}
