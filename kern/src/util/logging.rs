use core::fmt::Write;

use alloc::{fmt, format, string::ToString};
use jrinx_hal::{Cpu, Earlycon, Hal};
use jrinx_multitask::runtime;

use super::color::{self, with_color};

struct Logger;

impl Write for Logger {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for b in s.bytes() {
            hal!().earlycon().putc(b);
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
        let cpu_id = hal!().cpu().id();
        let cpu_time = hal!().cpu().get_time();
        let level = record.level();
        let color = match level {
            log::Level::Error => color::ColorCode::Red,
            log::Level::Warn => color::ColorCode::Yellow,
            log::Level::Info => color::ColorCode::Green,
            log::Level::Debug => color::ColorCode::Cyan,
            log::Level::Trace => color::ColorCode::Magenta,
        };

        let kernel_state = runtime::with_current(|rt| {
            rt.with_current_inspector(|inspector| {
                inspector
                    .with_current_executor(|executor| format!("executor#{}", executor.id()))
                    .unwrap_or(format!("inspector#{}", inspector.id()))
            })
            .unwrap_or("runtime".to_string())
        })
        .unwrap_or("bootstrap".to_string());

        fmt::format(*record.args()).split('\n').for_each(|args| {
            print_fmt(with_color! {
                color::ColorCode::White,
                color::ColorCode::White,
                "[ {time} cpu#{id} {level} ] ( {kernel_state} ) {args}\n",
                time = {
                    let micros = cpu_time.as_micros();
                    format_args!("{s:>6}.{us:06}", s = micros / 1000000, us = micros % 1000000)
                },
                id = cpu_id,
                level = with_color!(color, color::ColorCode::White, "{:>5}", level),
                kernel_state = with_color!(color::ColorCode::Blue, color::ColorCode::White, "{:^14}", kernel_state),
                args = with_color!(color::ColorCode::White, color::ColorCode::White, "{}", args),
            });
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
