#![no_std]
#![no_main]
#![feature(format_args_nl)]
#![feature(alloc_error_handler)]

extern crate alloc;
#[macro_use]
extern crate rpstdlib;

use alloc::vec::Vec;

#[no_mangle]
fn main(arg: Vec<&'static str>) -> i32 {
  if arg.len() != 2 {
    println!("usage: write FILE TEXT...");
    return 0;
  }
  let path = arg[0];
  let file = rpstdlib::fs::File::create(path);
  match file {
    Ok(mut file) => {
      file.write(arg[1].as_bytes()).expect("write file failed");
      file.write(&[b'\n']).expect("write file failed");
    }
    Err(e) => {
      println!("{}", e);
    }
  }
  rpstdlib::exit();
}
