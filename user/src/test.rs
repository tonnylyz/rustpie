#![no_std]
#![no_main]
#![feature(format_args_nl)]
#![feature(alloc_error_handler)]

extern crate alloc;
extern crate rlibc;
#[macro_use]
extern crate exported;
extern crate microcall;

use alloc::vec::Vec;
use fs::File;
use microcall::message::Message;

#[no_mangle]
fn _start(arg: *const u8) -> ! {
  // microcall::null();
  let msg = Message::default();
  msg.call(common::server::SERVER_TEST).unwrap();
  loop {}
}
