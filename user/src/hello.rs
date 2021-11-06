#![no_std]
#![no_main]
#![feature(format_args_nl)]
#![feature(alloc_error_handler)]

extern crate rlibc;

#[no_mangle]
fn _start(_arg: *const u8) -> ! {
  exported::exit();
}
