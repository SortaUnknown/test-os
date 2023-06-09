use spin::{Mutex, Lazy};
use uart_16550::SerialPort;
use core::fmt::{Arguments, Write};

static SERIAL1: Lazy<Mutex<SerialPort>> = Lazy::new(||
{
    let mut serial_port = unsafe{SerialPort::new(0x3F8)};
    serial_port.init();
    Mutex::new(serial_port)
});

#[doc(hidden)]
pub fn _print(args: Arguments)
{
    x86_64::instructions::interrupts::without_interrupts(|| {SERIAL1.lock().write_fmt(args).expect("Printing to serial failed");});
}

#[macro_export]
macro_rules! serial_print
{
    ($($arg:tt)*) => {$crate::serial::_print(format_args!($($arg)*));};
}

#[macro_export]
macro_rules! serial_println
{
    () => ($crate::serial_print!("\n"));
    ($fmt:expr) => ($crate::serial_print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::serial_print!(concat!($fmt, "\n"), $($arg)*));
}