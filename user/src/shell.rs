#![no_std]
#![no_main]
#![feature(format_args_nl)]
#![feature(alloc_error_handler)]

extern crate alloc;
#[macro_use]
extern crate exported;
extern crate rlibc;

#[no_mangle]
fn _start() -> ! {
  exported::heap::init();
  println!("Welcome to rustpi shell!");
  loop {
    print!("SHELL> ");
    let cmd = exported::stdio::getline();
    println!();
    if cmd.trim().is_empty() {
      continue;
    }
    match exported::pm::exec(cmd.as_str()) {
      Ok(pid) => {
        exported::pm::wait(pid);
      }
      Err(e) => {
        println!("exec failed: {}", e);
      }
    }
  }
}

