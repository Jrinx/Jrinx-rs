#![no_std]

use core::fmt::Write;
use spin::Mutex;

use jrinx_abi::sysfn;

struct Logger;

impl Write for Logger {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let bytes = s.as_bytes();
        let len = bytes.len();
        sysfn::sys_debug_log(bytes.as_ptr(), len);
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

        let mutex = MUTEX.lock();
        Logger.write_fmt(format_args!("{}", record.args())).unwrap();
        core::hint::black_box(mutex);
    }

    fn flush(&self) {}
}

pub fn init() {
    log::set_logger(&Logger).unwrap();
    log::set_max_level(log::LevelFilter::Debug);
}
