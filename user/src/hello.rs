#![no_std]
#![no_main]
#![feature(format_args_nl)]
#![feature(alloc_error_handler)]



#[no_mangle]
fn _start(_arg: *const u8) {
  // let arg = exported::parse(arg);
  exported::exit();
}
