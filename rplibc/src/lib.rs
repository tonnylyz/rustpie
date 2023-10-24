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

#[no_mangle]
extern "C" fn open(path: *const u8, flags: isize) -> i32 {
  match rpstdlib::fs::File::open_with_flags(
    core::str::from_utf8(unsafe { core::slice::from_raw_parts(path, strlen(path)) }).unwrap(),
    flags,
  ) {
    Ok(f) => {
      let fd = f.handle();
      core::mem::forget(f);
      fd as i32
    }
    Err(e) => e.errno,
  }
}

#[no_mangle]
extern "C" fn read(fd: i32, buf: *mut u8, cnt: usize) -> usize {
  let mut f = rpstdlib::fs::File::from_handle(fd as usize);
  let r = match f.read(unsafe { core::slice::from_raw_parts_mut(buf, cnt) }) {
    Ok(len) => len,
    Err(_e) => 0,
  };
  core::mem::forget(f);
  r
}

#[no_mangle]
extern "C" fn write(fd: i32, buf: *const u8, cnt: usize) -> usize {
  let mut f = rpstdlib::fs::File::from_handle(fd as usize);
  let r = match f.write(unsafe { core::slice::from_raw_parts(buf, cnt) }) {
    Ok(len) => len,
    Err(_e) => 0,
  };
  core::mem::forget(f);
  r
}

#[no_mangle]
extern "C" fn close(fd: i32) -> i32 {
  let f = rpstdlib::fs::File::from_handle(fd as usize);
  drop(f);
  0
}
