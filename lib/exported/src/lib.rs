#![no_std]
#![feature(panic_info_message)]

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::print::print_arg(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => (print!("\n"));
    ($($arg:tt)*) => ({
        $crate::print::print_arg(format_args_nl!($($arg)*));
    })
}

pub mod print;