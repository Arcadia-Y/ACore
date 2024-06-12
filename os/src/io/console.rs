use crate::drivers::uart::UART;
use core::fmt::{self, Write};
use crate::task::suspend_current_and_run_next;
struct Stdout;

impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            UART.putc(c as u8)
        }
        Ok(())
    }
}

pub fn print(args: fmt::Arguments) {
    Stdout.write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::io::console::print(format_args!($fmt $(, $($arg)+)?));
    }
}

#[macro_export]
macro_rules! println {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::io::console::print(format_args!(concat!($fmt, "\n") $(, $($arg)+)?));
    }
}

// keep polling to get a char from UART
pub fn getchar() -> u8 {
    loop {
        if let Some(c) = UART.getc() {
            return c;
        }
        suspend_current_and_run_next();
    }
}
