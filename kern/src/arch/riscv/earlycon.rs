pub fn putc(ch: u8) {
    sbi::legacy::console_putchar(ch);
}
