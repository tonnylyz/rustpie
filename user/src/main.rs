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
    let c = exported::stdio::getline();
    println!("{}", c);
    exported::pm::exec(c.as_str(), 0);
  }
}

// #[no_mangle]
// fn _start() -> ! {
//   println!("bbbbbbbbbbbbbbbbb");
//   loop {
//     exported::sched_yield();
//   }
// }

