#![no_std]
#![no_main]
#![feature(format_args_nl)]

extern crate rlibc;
#[macro_use]
extern crate exported;

use exported::getchar;

#[no_mangle]
fn _start() -> ! {
    println!("User image!");
    loop {
        let c = getchar();
        print!("{}", c);
    }
}
