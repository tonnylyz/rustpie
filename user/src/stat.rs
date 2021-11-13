#![no_std]
#![no_main]
#![feature(format_args_nl)]
#![feature(alloc_error_handler)]

extern crate alloc;
extern crate rlibc;
#[macro_use]
extern crate exported;

use fs::File;

#[no_mangle]
fn _start(arg: *const u8) {
  let arg = exported::parse(arg);
  if arg.len() == 0 {
    println!("usage: stat FILE...");
    exported::exit();
  }
  let path = arg[0];
  let file = File::open(path).expect("open file failed");

  match file.stat() {
    Ok(stat) => {
      println!("{:#?}", stat);
    }
    Err(e) => {
      println!("{}", e);
    }
  }
  exported::exit();
}
