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

pub fn exit() -> ! {
  microcall::thread_destroy(0).unwrap();
  loop {}
}

#[panic_handler]
fn panic_handler(info: &core::panic::PanicInfo) -> ! {
  let asid = microcall::get_asid(0).unwrap();
  if let Some(m) = info.message() {
    if let Some(l) = info.location() {
      println!("[USER][panic] p{} {} \n {}", asid, m, l);
    } else {
      println!("[USER][panic] p{} {}", asid, m);
    }
  } else {
    println!("[USER][panic] p{} no message", asid);
  }
  exit()
}
