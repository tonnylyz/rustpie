#![no_std]
#![no_main]
#![feature(format_args_nl)]
#![feature(alloc_error_handler)]

extern crate alloc;
extern crate rlibc;
#[macro_use]
extern crate exported;

#[no_mangle]
fn _start() -> ! {
  exported::pm::ps();
  exported::exit()
}
