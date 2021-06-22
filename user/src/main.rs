#![no_std]
#![no_main]
#![feature(format_args_nl)]

extern crate rlibc;
#[macro_use]
extern crate exported;

#[no_mangle]
fn _start() -> ! {
    println!("User image!");
    loop {}
}
