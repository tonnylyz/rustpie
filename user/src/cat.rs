#![no_std]
#![no_main]

extern crate alloc;
#[macro_use]
extern crate rpstdlib;

use alloc::vec::Vec;

#[no_mangle]
fn main(arg: Vec<&'static str>) -> i32 {
  for file in arg {
    let path = file;
    use rpstdlib::fs::File;
    let mut file = File::open(path).expect("cannot open file");
    let mut buf = [0u8; 128];
    loop {
      let read = file.read(&mut buf).unwrap();
      let str = core::str::from_utf8(&buf[0..read]).unwrap();
      print!("{}", str);
      if read < 128 {
        break;
      }
    }
  }
  0
}
