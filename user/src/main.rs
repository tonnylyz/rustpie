#![no_std]
#![no_main]
#![feature(global_asm)]
#![feature(llvm_asm)]
#![feature(alloc_error_handler)]
#![feature(panic_info_message)]
#![feature(format_args_nl)]
#![feature(box_syntax)]

#[macro_use]
extern crate alloc;
extern crate rlibc;

#[macro_use]
mod print;

#[cfg(target_arch = "aarch64")]
#[path = "arch/aarch64/mod.rs"]
mod arch;

#[cfg(target_arch = "riscv64")]
#[path = "arch/riscv64/mod.rs"]
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
mod traits;
#[allow(dead_code)]
mod microcall;
mod thread;
mod thread_sys;
mod thread_parker;

#[no_mangle]
fn _start(arg: usize) -> ! {
  page_fault::init();
  heap::init();
  match arg {
    0 => { fork::test() }
    1 => { virtio_blk::server() }
    _ => {}
  }
  microcall::thread_destroy(0);
  loop {};
}
