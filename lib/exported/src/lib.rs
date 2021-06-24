#![no_std]
#![feature(panic_info_message)]
#![feature(format_args_nl)]
#![feature(alloc_error_handler)]

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;

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
pub mod heap;
pub mod mm;

pub fn getchar() -> u8 {
  let server_tid = microcall::server_tid_wait(common::server::SERVER_TERMINAL);
  loop {
    match microcall::itc_call(server_tid, 0, 0, 0, 0) {
      Ok((_, c, _, _, _)) => {
        if c != 0 {
          break c as u8;
        }
        microcall::thread_yield()
      }
      Err(_) => {
        microcall::thread_yield()
      }
    }
  }
}

pub fn getline() -> String {
  let mut v = Vec::new();
  loop {
    let c = getchar();
    v.push(c);
    if c == 0xd {
      break
    }
  }
  String::from_utf8(v).expect("getline failed!")
}