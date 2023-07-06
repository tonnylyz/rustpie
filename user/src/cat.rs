#![no_std]
#![no_main]
#![feature(format_args_nl)]
#![feature(alloc_error_handler)]

extern crate alloc;
#[macro_use]
extern crate rpstdlib;


#[no_mangle]
fn _start(arg: *const u8) {
  let arg = rpstdlib::parse(arg);
  for file in arg {
    let path = file;
    use rpstdlib::fs::File;
    let mut file = File::open(path).expect("cannot open file");
    let mut buf = [0u8; 128];
    loop {
      let read = file.read(&mut buf).unwrap();
      let str = core::str::from_utf8(&buf[0..read]).unwrap();
      print!("{}", str);
      if read < 128 {
        break;
      }
    }
  }
  rpstdlib::exit();
}
