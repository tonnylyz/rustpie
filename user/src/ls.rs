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
  let path = if arg.len() == 0 {
    "/"
  } else {
    arg[0]
  };
  let mut root = fs::File::open(path).unwrap();
  let mut buf = [0u8; 128];
  root.read(&mut buf).unwrap();
  let dir = core::str::from_utf8(&buf).unwrap();
  for c in dir.chars() {
    if c == '\n' {
      print!("\t");
    } else {
      print!("{}", c);
    }
  }
  println!();
  exported::exit();
}
