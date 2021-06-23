#![no_std]
#![feature(panic_info_message)]
#![feature(format_args_nl)]

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

pub fn getchar() -> char {
  let server_tid = microcall::server_tid_wait(common::server::SERVER_TERMINAL);
  loop {
    match microcall::itc_call(server_tid, 0, 0, 0, 0) {
      Ok((_, c, _, _, _)) => {
        if c != 0 {
          break c as u8 as char;
        }
        microcall::thread_yield()
      }
      Err(_) => {
        microcall::thread_yield()
      }
    }
  }
}
