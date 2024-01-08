#![no_std]

extern crate alloc;

use core::fmt::Write;

use alloc::{
    fmt, format,
    string::{String, ToString},
};
use jrinx_error::InternalError;
use jrinx_hal::{hal, Cpu, Earlycon, Hal, Interrupt};
use jrinx_multitask::{
    executor::Executor,
    inspector::Inspector,
    runtime::{Runtime, RuntimeStatus},
};
use jrinx_util::color;
use spin::Mutex;

#[cfg(feature = "colorful")]
macro_rules! with_color {
    ($color:expr, $restore_color:expr, $($arg:tt)*) => {
        format_args!(
            "\u{1B}[{color}m{arg}\u{1B}[{restore}m",
            color = $color as u8,
            arg = format_args!($($arg)*),
            restore = $restore_color as u8,
        )
    };
}

#[cfg(not(feature = "colorful"))]
macro_rules! with_color {
    ($color:expr, $restore_color:expr, $($arg:tt)*) => {{
        $color as u8;
        $restore_color as u8;
        format_args!($($arg)*)
    }};
}

struct Logger;

impl Write for Logger {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for b in s.bytes() {
            hal!().earlycon().putc(b);
        }
        Ok(())
    }
}

impl log::Log for Logger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::max_level()
    }

    fn log(&self, record: &log::Record) {
        static MUTEX: Mutex<()> = Mutex::new(());
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

        let kernel_state = analyse_kernel_state();
        fmt::format(*record.args()).split('\n').for_each(|args| {
            hal!().interrupt().with_saved_off(|| {
                let mutex = MUTEX.lock();
                Logger.write_fmt(with_color! {
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
                }).unwrap();
                core::hint::black_box(mutex);
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

pub fn set_max_level(level: log::LevelFilter) {
    log::set_max_level(level);
}

fn analyse_kernel_state() -> String {
    if let Ok(state) = match Executor::with_current(|ex| ex.id()) {
        Ok(id) => Ok(format!("executor#{}", id)),
        Err(err) => Err(err),
    } {
        state
    } else if let Ok(state) = match Inspector::with_current(|is| is.id()) {
        Ok(id) => Ok(format!("inspector#{}", id)),
        Err(err) => Err(err),
    } {
        state
    } else if let Ok(state) = match Runtime::with_current(|rt| rt.status()) {
        RuntimeStatus::Unused => Err(InternalError::InvalidRuntimeStatus),
        _ => Ok("runtime".to_string()),
    } {
        state
    } else {
        "bootstrap".to_string()
    }
}
