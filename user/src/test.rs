#![no_std]
#![no_main]
#![feature(format_args_nl)]
#![feature(alloc_error_handler)]
#![feature(asm)]

extern crate alloc;
#[macro_use]
extern crate exported;
extern crate microcall;
extern crate rlibc;

use microcall::message::Message;

#[allow(dead_code)]
#[cfg(target_arch = "aarch64")]
pub fn current_cycle() -> usize {
  let r;
  unsafe {
    asm!("mrs {}, pmccntr_el0", out(reg) r);
  }
  r
}

#[allow(dead_code)]
#[cfg(target_arch = "riscv64")]
pub fn current_cycle() -> usize {
  let r;
  unsafe {
    asm!("rdcycle {}", out(reg) r);
  }
  r
}

#[allow(dead_code)]
fn test_null_syscall() {
  for _ in 0..1000 {
    let icntr = current_cycle();
    microcall::null();
    let icntr2 = current_cycle();
    println!("{}", icntr2 - icntr);
  }
  exported::exit();
}

#[allow(dead_code)]
fn test_as_switch() {
  for _ in 0..1000 {
    let icntr = current_cycle();
    microcall::thread_yield();
    let icntr2 = current_cycle();
    println!("{}", icntr2 - icntr);
  }
  exported::exit();
}

#[no_mangle]
fn _start(_arg: *const u8) -> ! {
  // test_null_syscall();
  // test_as_switch();
  let mut msg = Message::default();
  for _ in 0..1000 {
    let icntr = current_cycle();
    msg.a = icntr;
    let _ = msg.call(common::server::SERVER_TEST);
  }
  exported::exit();
}
