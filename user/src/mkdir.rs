#![no_std]
#![no_main]
#![feature(format_args_nl)]
#![feature(alloc_error_handler)]

extern crate alloc;
extern crate rlibc;
#[macro_use]
extern crate exported;

#[no_mangle]
fn _start(arg: *const u8) {
  let arg = exported::parse(arg);
  if arg.len() != 1 {
    println!("usage: mkdir DIRECTORY...");
    exported::exit();
  }
  let path = arg[0];
  match fs::create_dir(path) {
    Ok(_) => {}
    Err(e) => {
      println!("{}", e);
    }
  }
  exported::exit();
}
