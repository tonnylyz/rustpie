#![no_std]
#![no_main]
#![feature(format_args_nl)]
#![feature(alloc_error_handler)]

extern crate alloc;
#[macro_use]
extern crate exported;


#[no_mangle]
fn _start(arg: *const u8) {
  let arg = exported::parse(arg);
  if arg.len() == 0 {
    println!("usage: rd DIR...");
    exported::exit();
  }
  let path = arg[0];
  match fs::remove_directory(path) {
    Ok(_) => {}
    Err(e) => {
      println!("{}", e);
    }
  }
  exported::exit();
}
