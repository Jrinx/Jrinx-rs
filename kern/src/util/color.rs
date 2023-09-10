#[allow(dead_code)]
#[repr(u8)]
pub enum ColorCode {
    RED = 31,
    GREEN = 32,
    YELLOW = 33,
    MAGENTA = 35,
    CYAN = 36,
    White = 37,
}

macro_rules! with_color {
    ($color:expr, $($arg:tt)*) => {{
        #[cfg(feature = "colorful")]
        {
            format_args!("\u{1B}[{}m{}\u{1B}[0m", $color as u8, format_args!($($arg)*))
        }

        #[cfg(not(feature = "colorful"))]
        {
            $color as u8;
            format_args!($($arg)*)
        }
    }};
}
pub(crate) use with_color;
