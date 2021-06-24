#![no_std]
#![no_main]
#![feature(format_args_nl)]
#![feature(alloc_error_handler)]

extern crate rlibc;
#[macro_use]
extern crate exported;

#[no_mangle]
fn _start() -> ! {
  exported::heap::init();
  println!("User image!");
  loop {
    let c = exported::getline();
    println!("{}", c);
  }
}
