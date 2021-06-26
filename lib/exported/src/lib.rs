#![no_std]
#![feature(panic_info_message)]
#![feature(format_args_nl)]
#![feature(alloc_error_handler)]

extern crate alloc;

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::stdio::print_arg(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => (print!("\n"));
    ($($arg:tt)*) => ({
        $crate::stdio::print_arg(format_args_nl!($($arg)*));
    })
}

pub mod heap;
pub mod mm;
pub mod pm;
pub mod stdio;

pub fn sched_yield() {
  microcall::thread_yield();
}

#[panic_handler]
fn panic_handler(_info: &core::panic::PanicInfo) -> ! {
  loop {}
}
