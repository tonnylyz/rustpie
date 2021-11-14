#![no_std]
#![no_main]
#![feature(format_args_nl)]
#![feature(alloc_error_handler)]

extern crate alloc;
#[macro_use]
extern crate exported;
extern crate rlibc;

use alloc::string::ToString;
use alloc::vec::Vec;

use getopts::Options;

#[no_mangle]
fn _start(arg: *const u8) {
  let args = exported::parse(arg);
  main(args);
  exported::exit();
}

fn usage(opts: Options) {
  print!("{}", opts.usage("Usage: touch FILE [options]"));
}

fn main(args: Vec<&str>) {
  let mut opts = Options::new();
  opts.optflag("h", "help", "display this help and exit");
  let matches = match opts.parse(args) {
    Ok(m) => { m }
    Err(f) => { panic!("{}", f.to_string()); }
  };
  if matches.opt_present("h") {
    usage(opts);
    return;
  }
  let input = if !matches.free.is_empty() {
    matches.free[0].clone()
  } else {
    usage(opts);
    return;
  };

  match fs::File::create(input) {
    Ok(_) => {}
    Err(e) => {
      println!("{}", e);
    }
  }
}