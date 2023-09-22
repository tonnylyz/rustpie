#![no_std]
#![no_main]

extern crate alloc;

use rpstdlib::println;

use alloc::vec::Vec;

#[no_mangle]
fn main(_arg: Vec<&'static str>) -> i32 {
  println!("hello world!");
  0
}
