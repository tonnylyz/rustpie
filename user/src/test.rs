#![no_std]
#![no_main]
#![feature(format_args_nl)]
#![feature(alloc_error_handler)]
#![feature(asm)]

#[macro_use]
extern crate alloc;
#[macro_use]
extern crate exported;
extern crate microcall;


use spin::Once;
use microcall::message::Message;
use microcall::yield_to;

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
  let mut results = vec![];
  for _ in 0..1000 {
    let icntr = current_cycle();
    microcall::null(0);
    let icntr2 = current_cycle();
    results.push(icntr2 - icntr);
  }
  let mut sum = 0;
  for result in results {
    sum += result;
  }
  println!("[[TEST]] test_null_syscall {}/1000", sum);
  exported::exit();
}

fn switch_back(test_thread: usize) {
  loop {
    yield_to(test_thread);
  }
}

#[allow(dead_code)]
fn test_thread_switch() {
  exported::mm::page_alloc(0x10000000);
  let child_thread = microcall::thread_alloc(0, switch_back as usize, 0x10001000, microcall::get_tid()).unwrap();
  let mut results = vec![];
  for _ in 0..1000 {
    let icntr = current_cycle();
    yield_to(child_thread);
    let icntr2 = current_cycle();
    results.push(icntr2 - icntr);
  }
  let mut sum = 0;
  for result in results {
    sum += result;
  }
  println!("[[TEST]] test_thread_switch {}/1000", sum);
  println!("after this test, system cannot continue, please restart");
  loop {}
}

#[allow(dead_code)]
fn test_ipc_call() {
  let msg = Message::default();
  let mut results = vec![];
  for _ in 0..1000 {
    let icntr = current_cycle();
    let r = msg.call(common::server::SERVER_TEST).unwrap();
    let icntr2 = r.a;
    results.push(icntr2 - icntr);
    println!("{}", icntr2 - icntr);
  }
  let mut sum = 0;
  for result in results {
    sum += result;
  }
  println!("[[TEST]] test_call {}/1000", sum);
  exported::exit();
}

#[no_mangle]
fn _start(arg: *const u8) -> ! {
  let arg = exported::parse(arg);
  if arg.len() == 0 {
    println!("usage: test {{ null | call | sw | kp | kf | sp | sf }}");
    exported::exit();
  }
  println!("test user program t{}", microcall::get_tid());
  match arg[0] {
    "null" => { // test null call
      test_null_syscall();
    }
    "call" => { // test IPC call
      test_ipc_call();
    }
    "sw" => { // test switch thread
      test_thread_switch();
    }
    "kp" => { // test kernel panic
      microcall::null(1);
    }
    "kf" => { // test kernel page fault
      microcall::null(2);
    }
    "sp" => { // test server panic
      let mut msg = Message::default();
      msg.a = 1;
      let _ = msg.call(common::server::SERVER_TEST).unwrap();
    }
    "sf" => { // test server page fault
      let mut msg = Message::default();
      msg.a = 2;
      let _ = msg.call(common::server::SERVER_TEST).unwrap();
    }
    _ => {
      println!("usage: test {{ null | call | sw | kp | kf | sp | sf }}")
    }
  }
  exported::exit();
}
