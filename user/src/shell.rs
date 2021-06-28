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
  exported::heap::init();
  println!("Welcome to rustpi shell!");
  loop {
    print!("SHELL> ");
    let cmd = exported::stdio::getline();
    println!();
    if let Ok(pid) = exported::pm::exec(cmd.as_str()) {
      exported::pm::wait(pid);
    }
  }
}
