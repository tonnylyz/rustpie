#![no_std]
#![no_main]
// #![feature(format_args_nl)]
#![feature(alloc_error_handler)]

extern crate alloc;
#[macro_use]
extern crate rpstdlib;

use alloc::vec::Vec;

#[no_mangle]
fn main(arg: Vec<&'static str>) -> i32 {
  if arg.len() != 1 {
    println!("usage: mkdir DIRECTORY...");
    return 0;
  }
  let path = arg[0];
  match rpstdlib::fs::create_dir(path) {
    Ok(_) => {}
    Err(e) => {
      println!("{}", e);
    }
  }
  0
}
