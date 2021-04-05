#![no_std]
#![no_main]
#![feature(global_asm)]
#![feature(llvm_asm)]
#![feature(alloc_error_handler)]
#![feature(panic_info_message)]
#![feature(format_args_nl)]

extern crate alloc;
extern crate rlibc;

use arch::page_table::*;
use fork::*;
use ipc::*;
use page_fault::*;
use syscall::*;

use crate::config::PAGE_SIZE;
use crate::arch::Address;

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
mod virtio_blk;


#[no_mangle]
fn _start(arg: usize) -> ! {
  set_page_fault_handler(page_fault_handler as usize);
  // set_self_ipc(getpid());
  heap::init();
  match arg {
    0 => { fktest() }
    1 => { pingpong() }
    2 => { heap_test() }
    3 => { virtio_blk() }
    _ => unsafe { print(arg as u8 as char) }
  }
  match process_destroy(0) {
    Ok(_) => {}
    Err(_) => {}
  }
  loop {};
}

fn virtio_blk_irq() {
  panic!("virtio_blk_irq");
}

fn virtio_blk() {
  virtio_blk::init();
  println!("virtio_blk init ok");
  mem_alloc(0, 0x7_0000_0000, PTE_DEFAULT);
  process_set_exception_handler(0, virtio_blk_irq as usize, 0x7_0000_1000, 0x10 + 32);
  mem_alloc(0, 0x3000_0000, PTE_DEFAULT).unwrap();
  virtio_blk::read(0, 8, 0x3000_0000);
  // let slice = unsafe { core::slice::from_raw_parts(0x3000_0000 as *const u8, PAGE_SIZE) };
  // for i in 0..4096 {
  //   print!("{:02x} ", slice[i]);
  //   if (i + 1) % 16 == 0 {
  //     println!();
  //   }
  // }


  loop {}
}

fn pingpong() {
  let who = fork();
  if who > 0 {
    println!("send 0 from {} to {}", get_asid(), who);
    ipc::send(who as u16, 0, 0, PTE_DEFAULT);
  }
  loop {
    println!("{} is waiting", get_asid());
    let (who, value, _) = ipc::receive(0);
    println!("{} received {} from {}", get_asid(), value, who);
    if value == 10 {
      return;
    }
    let value = value + 1;
    println!("{} send {} to {}", get_asid(), value, who);
    ipc::send(who, value, 0, PTE_DEFAULT);
    if value == 10 {
      return;
    }
  }
}

fn fktest() {
  println!("fktest started pid {}", get_asid());
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
      llvm_asm!("nop");
    }
  }
}