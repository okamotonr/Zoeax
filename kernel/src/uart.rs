use core::fmt;

use crate::sbi;

pub fn putchar(ch: u8) {
    sbi::sbi_call(ch as i64, 0, 0, 0, 0, 0, 0, 1);
}

pub struct Uart;


impl fmt::Write for Uart {
    fn write_str(&mut self, s: &str) -> fmt::Result {
       for ch in s.as_bytes() {
           putchar(*ch)
       } 

       Ok(())
    }
}


#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    let mut uart = Uart;
    use fmt::Write;
    uart.write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::uart::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

