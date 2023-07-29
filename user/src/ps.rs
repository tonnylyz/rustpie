#![no_std]
#![no_main]
#![feature(format_args_nl)]
#![feature(alloc_error_handler)]

extern crate alloc;
extern crate rpstdlib;

use alloc::vec::Vec;
#[no_mangle]
fn main(_arg: Vec<&'static str>) -> i32 {
  rpstdlib::pm::ps();
  0
}
