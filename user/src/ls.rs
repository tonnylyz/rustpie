#![no_std]
#![no_main]

extern crate alloc;
#[macro_use]
extern crate rpstdlib;

use alloc::vec::Vec;

#[no_mangle]
fn main(arg: Vec<&'static str>) -> i32 {
  let path = if arg.len() == 0 {
    "/"
  } else {
    arg[0]
  };
  let mut root = rpstdlib::fs::File::open(path).unwrap();
  let mut buf = [0u8; 128];
  root.read(&mut buf).unwrap();
  let dir = core::str::from_utf8(&buf).unwrap();
  for c in dir.chars() {
    if c == '\n' {
      print!("\t");
    } else {
      print!("{}", c);
    }
  }
  println!();
  0
}
