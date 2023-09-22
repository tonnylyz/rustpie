#![no_std]
#![no_main]

extern crate alloc;
extern crate rpstdlib;

use alloc::vec::Vec;
#[no_mangle]
fn main(_arg: Vec<&'static str>) -> i32 {
  rpstdlib::pm::ps();
  0
}
