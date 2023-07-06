#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::libtrusted::print::print_arg(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => (print!("\n"));
    ($($arg:tt)*) => ({
        $crate::libtrusted::print::print_arg(format_args_nl!($($arg)*));
    })
}

pub mod print;

#[allow(dead_code)]
pub mod thread;
pub mod mm;
pub mod loader;
pub mod foreign_slice;
pub mod wrapper;
pub mod exception;
