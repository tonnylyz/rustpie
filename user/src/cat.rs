#![no_std]
#![no_main]
#![feature(format_args_nl)]
#![feature(alloc_error_handler)]

extern crate alloc;
extern crate rlibc;
#[macro_use]
extern crate exported;

use alloc::vec::Vec;

#[inline(always)]
pub fn round_up(addr: usize, n: usize) -> usize {
  (addr + n - 1) & !(n - 1)
}

#[no_mangle]
fn _start(arg: *const u8) -> ! {
  exported::heap::init();
  let mut arguments = Vec::new();
  unsafe {
    let cmd = core::slice::from_raw_parts(arg, round_up(arg as usize, 4096) - arg as usize - 1);
    let cmd = core::str::from_utf8(cmd).unwrap();
    let mut iter = cmd.split_ascii_whitespace();
    loop {
      if let Some(arg) = iter.next() {
        arguments.push(arg);
      } else {
        break;
      }
    }
  }

  main(arguments);
  exported::exit();
}

fn main(arg: Vec<&str>) {
  for file in arg {
    let path = file;
    use fs::File;
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
}
