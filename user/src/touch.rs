#![no_std]
#![no_main]
#![feature(format_args_nl)]
#![feature(alloc_error_handler)]

extern crate alloc;
#[macro_use]
extern crate rpstdlib;


use alloc::string::ToString;
use alloc::vec::Vec;

use getopts::Options;

fn usage(opts: Options) {
  print!("{}", opts.usage("Usage: touch FILE [options]"));
}

#[no_mangle]
fn main(arg: Vec<&'static str>) -> i32 {
  let mut opts = Options::new();
  opts.optflag("h", "help", "display this help and exit");
  let matches = match opts.parse(arg) {
    Ok(m) => { m }
    Err(f) => { panic!("{}", f.to_string()); }
  };
  if matches.opt_present("h") {
    usage(opts);
    return 0;
  }
  let input = if !matches.free.is_empty() {
    matches.free[0].clone()
  } else {
    usage(opts);
    return 0;
  };

  match rpstdlib::fs::File::create(input) {
    Ok(_) => {}
    Err(e) => {
      println!("{}", e);
    }
  }
  0
}