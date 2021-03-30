#![feature(asm)]
#![feature(global_asm)]
#![feature(panic_info_message)]
#![feature(format_args_nl)]
#![feature(core_intrinsics)]
#![feature(alloc_error_handler)]
#![no_std]
#![no_main]

extern crate rlibc;
extern crate alloc;

use fork::*;
use ipc::*;
use page_fault::*;
use arch::page_table::*;
use syscall::*;
use crate::config::PAGE_SIZE;

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::print::print_arg(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ({
        $crate::print::print_arg(format_args_nl!($($arg)*));
    })
}

mod arch;
mod config;
mod print;
mod page_fault;
mod syscall;
mod fork;
mod ipc;
mod heap;


#[no_mangle]
fn _start(arg: usize) -> ! {
  set_page_fault_handler(page_fault_handler as usize);
  // set_self_ipc(getpid());
  heap::init();
  match arg {
    0 => { fktest() }
    1 => { pingpong() }
    2 => { heap_test() }
    3 => { main() }
    _ => unsafe { print(arg as u8 as char) }
  }
  match process_destroy(0) {
    Ok(_) => {},
    Err(_) => {},
  }
  loop {};
}

fn print2() { loop { print!("2") } }

fn main() {
  println!("je;;p");
  mem_alloc(0, 0x7000_0000 - PAGE_SIZE, PTE_W);
  println!("je;;p");
  thread_alloc(0x12345678, 0x7000_0000, 0);

  loop { print!("1") }

}

fn pingpong() {
  let who = fork();
  if who > 0 {
    println!("send 0 from {} to {}", getpid(), who);
    ipc::send(who as u16, 0, 0, PTE_DEFAULT);
  }
  loop {
    println!("{} is waiting", getpid());
    let (who, value, _) = ipc::receive(0);
    println!("{} received {} from {}", getpid(), value, who);
    if value == 10 {
      return;
    }
    let value = value + 1;
    println!("{} send {} to {}", getpid(), value, who);
    ipc::send(who, value, 0, PTE_DEFAULT);
    if value == 10 {
      return;
    }
  }
}

fn fktest() {
  println!("fktest started pid {}", getpid());
  let mut a = 0;
  let mut id = fork();
  if id == 0 {
    id = fork();
    if id == 0 {
      a += 3;
      loop {
        print!("{}", a);
      }
    }
    a += 2;
    loop {
      print!("{}", a);
    }
  }
  a += 1;
  loop {
    print!("{}", a);
  }
}

fn heap_test() {
  use alloc::vec::Vec;
  let mut a = Vec::new();
  a.push(1);
  a.push(2);
  a.push(3);
  let pid = fork();
  if pid == 0 {
    println!("child {}", a.len());
  } else {
    a.push(4);
    a.push(5);
    println!("parent {}", a.len());
  }
}

unsafe fn print(a: char) {
  loop {
    print!("{}", a);
    for _ in 0..0x1000000 {
      asm!("nop");
    }
  }
}