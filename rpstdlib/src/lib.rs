#![no_std]
#![feature(panic_info_message)]
#![feature(format_args_nl)]
#![feature(alloc_error_handler)]

extern crate alloc;

use alloc::vec::Vec;

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
pub mod rtc;
pub mod fs;

pub fn sched_yield() {
  rpsyscall::thread_yield();
}

#[inline(always)]
fn round_up(addr: usize, n: usize) -> usize {
  (addr + n - 1) & !(n - 1)
}

fn parse(arg: *const u8) -> Vec<&'static str> {
  heap::init();
  let mut arguments = Vec::new();
  let mut first = true;
  unsafe {
    let cmd = core::slice::from_raw_parts(arg, round_up(arg as usize, 4096) - arg as usize - 1);
    let cmd = core::str::from_utf8(cmd).unwrap();
    let mut iter = cmd.split_ascii_whitespace();
    loop {
      if let Some(arg) = iter.next() {
        if !first {
          arguments.push(arg);
        }
        first = false;
      } else {
        break;
      }
    }
  }
  arguments
}

pub fn exit() -> ! {
  let _ = rpsyscall::thread_destroy(0);
  loop {}
}

#[cfg(not(feature = "libc"))]
extern {
  fn main(arg: Vec<&'static str>) -> i32;
}

#[no_mangle]
#[cfg(not(feature = "libc"))]
extern "C" fn _start(arg: *const u8) {
  let arg = parse(arg);
  unsafe {
    main(arg);
  }
  exit();
}

#[panic_handler]
fn panic_handler(info: &core::panic::PanicInfo) -> ! {
  let asid = rpsyscall::get_asid(0).unwrap();
  if let Some(m) = info.message() {
    if let Some(l) = info.location() {
      println!("[USER][panic] asid{} {} \n {}", asid, m, l);
    } else {
      println!("[USER][panic] asid{} {}", asid, m);
    }
  } else {
    println!("[USER][panic] asid{} no message", asid);
  }
  exit()
}
