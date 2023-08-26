use core::fmt::Write;

struct Logger;

impl Write for Logger {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for b in s.bytes() {
            sbi::legacy::console_putchar(b);
        }
        Ok(())
    }
}

pub fn print_fmt(args: core::fmt::Arguments) {
    Logger.write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! println {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::util::logging::print_fmt(format_args!(concat!($fmt, "\n") $(, $($arg)+)?));
    };
}

#[macro_export]
macro_rules! info {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::util::logging::print_fmt(format_args!(
            "[ tick@{} hart#{} ] ",
            $crate::arch::cpu::time(),
            $crate::arch::cpu::id())
        );
        $crate::util::logging::print_fmt(format_args!(concat!($fmt, "\n") $(, $($arg)+)?));
    };
}
