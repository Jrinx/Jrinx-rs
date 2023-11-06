use crate::Earlycon;

#[derive(Debug, Clone, Copy)]
pub struct EarlyconImpl;

pub(crate) static EARLYCON: EarlyconImpl = EarlyconImpl;

impl Earlycon for EarlyconImpl {
    fn getc(&self) -> Option<u8> {
        sbi::legacy::console_getchar()
    }

    fn putc(&self, c: u8) {
        sbi::legacy::console_putchar(c);
    }
}
