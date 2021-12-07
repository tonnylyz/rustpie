#![no_std]
#![no_main]
#![feature(format_args_nl)]
#![feature(alloc_error_handler)]

use exported::println;

#[no_mangle]
fn _start(_arg: *const u8) {
  println!("hello world!");
  exported::exit();
}
