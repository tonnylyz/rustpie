#![no_std]
#![no_main]

extern crate alloc;

use rpstdlib::println;

use alloc::vec::Vec;

#[no_mangle]
fn main(_arg: Vec<&'static str>) -> i32 {
  use rpstdlib::rtc::*;
  println!("{}", RtcTime::from_timestamp(timestamp().unwrap()));
  0
}
