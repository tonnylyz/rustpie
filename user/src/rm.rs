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
  if arg.len() == 0 {
    println!("usage: rm FILE...");
    return;
  }
  let path = arg[0];
  match fs::remove_file(path) {
    Ok(_) => {}
    Err(e) => {
      println!("{}", e);
    }
  }
}
