#![no_std]
#![no_main]
#![feature(format_args_nl)]
#![feature(alloc_error_handler)]

extern crate rlibc;

#[no_mangle]
fn _start(arg: *const u8) {
  // let arg = exported::parse(arg);
  exported::exit();
}
