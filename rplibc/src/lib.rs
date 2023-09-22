#![no_std]

use alloc::vec::Vec;

extern crate alloc;

extern "C" {
    fn main(args: i32, argv: *const u8) -> i32;
}

#[inline(always)]
fn round_up(addr: usize, n: usize) -> usize {
  (addr + n - 1) & !(n - 1)
}

#[no_mangle]
extern "C" fn _start(arg: *const u8) {
    rpstdlib::heap::init();
    let mut arguments = Vec::new();
    unsafe {
      let cmd = core::slice::from_raw_parts(arg, round_up(arg as usize, 4096) - arg as usize - 1);
      let cmd = core::str::from_utf8(cmd).unwrap();
      let mut iter = cmd.split_ascii_whitespace();
      loop {
        if let Some(arg) = iter.next() {
          let cstr = alloc::ffi::CString::new(arg).expect("arg to cstring failed");
          arguments.push(cstr.into_raw());
        } else {
          break;
        }
      }
    }
    unsafe {
        main(arguments.len() as i32, arguments.as_ptr() as *const u8);
    }
    rpstdlib::exit();
}

#[no_mangle]
extern "C" fn putc(c: u8) {
    rpsyscall::putc(c as char);
}

#[no_mangle]
extern "C" fn getc() -> u8 {
    rpstdlib::stdio::getchar()
}

#[no_mangle]
extern "C" fn strlen(str: *const u8) -> usize {
    let mut i = 0;
    unsafe {
        loop {
            if str.add(i).read() == 0 {
                return i;
            }
            i += 1;
        }
    }
}
