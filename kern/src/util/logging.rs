use core::fmt::Write;

use alloc::{format, string::ToString};

use crate::{arch, cpudata};

use super::color::{self, with_color};

struct Logger;

impl Write for Logger {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for b in s.bytes() {
            arch::earlycon::putc(b);
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

impl log::Log for Logger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::max_level()
    }

    fn log(&self, record: &log::Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        let cpu_id = arch::cpu::id();
        let cpu_time = arch::cpu::time();
        let level = record.level();
        let color = match level {
            log::Level::Error => color::ColorCode::RED,
            log::Level::Warn => color::ColorCode::YELLOW,
            log::Level::Info => color::ColorCode::GREEN,
            log::Level::Debug => color::ColorCode::CYAN,
            log::Level::Trace => color::ColorCode::MAGENTA,
        };

        let kernel_state = cpudata::with_cpu_runtime(|rt| {
            rt.with_current_executor(|executor| format!("executor#{}", executor.id()).to_string())
                .unwrap_or("runtime".to_string())
        })
        .unwrap_or("bootstrap".to_string());

        print_fmt(with_color! {
            color::ColorCode::WHITE,
            color::ColorCode::WHITE,
            "[ {time} cpu#{id} {level} ] ( {kernel_state} ) {args}\n",
            time = {
                let micros = cpu_time.as_micros();
                format_args!("{s:>6}.{us:06}", s = micros / 1000000, us = micros % 1000000)
            },
            id = cpu_id,
            level = with_color!(color, color::ColorCode::WHITE, "{:>5}", level),
            kernel_state = with_color!(color::ColorCode::BLUE, color::ColorCode::WHITE, "{:^14}", kernel_state),
            args = with_color!(color::ColorCode::WHITE, color::ColorCode::WHITE, "{}", record.args()),
        });
    }

    fn flush(&self) {}
}

pub fn init() {
    static LOGGER: Logger = Logger;
    log::set_logger(&LOGGER).unwrap();
    if let Some(level) = option_env!("LOGLEVEL") {
        log::set_max_level(level.parse().unwrap());
    } else {
        log::set_max_level(log::LevelFilter::Info);
    }
}
