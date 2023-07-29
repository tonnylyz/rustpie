#![no_std]
#![no_main]
#![feature(format_args_nl)]
#![feature(alloc_error_handler)]

extern crate alloc;

use rpstdlib::println;

use alloc::vec::Vec;

#[no_mangle]
fn main(arg: Vec<&'static str>) -> i32 {
  println!("hello world!");
  0
}
