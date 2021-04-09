#![no_std]
#![no_main]
#![feature(global_asm)]
#![feature(llvm_asm)]
#![feature(alloc_error_handler)]
#![feature(panic_info_message)]
#![feature(format_args_nl)]

#[macro_use]
extern crate alloc;
extern crate rlibc;


#[macro_use]
mod print;

mod arch;
mod config;
mod page_fault;
#[allow(dead_code)]
mod syscall;
mod fork;
mod heap;
mod virtio_blk;
mod itc;
mod fs;
mod mem;

use crate::page_fault::{set_page_fault_handler, page_fault_handler};
use crate::syscall::thread_destroy;

#[no_mangle]
fn _start(arg: usize) -> ! {
  set_page_fault_handler(page_fault_handler as usize);
  heap::init();
  match arg {
    0 => { fork::test() }
    1 => { virtio_blk::server() }
    _ => {}
  }
  thread_destroy(0);
  loop {};
}
