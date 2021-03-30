#![no_std]
#![no_main]
#![feature(global_asm)]
#![feature(alloc_error_handler)]
#![feature(panic_info_message)]
#![feature(core_intrinsics)]
#![feature(core_panic)]
#![feature(format_args_nl)]
#![feature(llvm_asm)]
#![feature(lang_items)]

extern crate alloc;
extern crate rlibc;

use arch::*;

use crate::lib::current_thread;
use crate::lib::core::barrier;
use crate::panic::{init_backtrace, init_backtrace_context};

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::lib::print::print_arg(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ({
        $crate::lib::print::print_arg(format_args_nl!($($arg)*));
    })
}

#[lang = "eh_personality"]
#[no_mangle]
pub extern fn rust_eh_personality() {
  println!("rust_eh_personality called");
  loop {}
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern fn _Unwind_Resume() {
  println!("_Unwind_Resume called");
  loop {}
}

mod arch;
mod board;
mod driver;
mod lib;
mod mm;
mod config;
mod panic;

fn clear_bss() {
  extern "C" {
    fn BSS_START();
    fn BSS_END();
  }
  let start = (BSS_START as usize).pa2kva();
  let end = (BSS_END as usize).pa2kva();
  unsafe { core::intrinsics::volatile_set_memory(start as *mut u8, 0, end - start); }
}

fn static_check() {
  use core::mem::size_of;
  // Note: size of ContextFrame needs to be synced with `arch/*/exception.S`
  if cfg!(target_arch = "aarch64") {
    assert_eq!(size_of::<ContextFrame>(), 0x110);
  } else if cfg!(target_arch = "riscv64") {
    assert_eq!(size_of::<ContextFrame>(), 0x110);
  } else {
    panic!("unsupported arch");
  }
}

#[no_mangle]
pub unsafe fn main(core_id: CoreId) -> ! {
  if core_id == 0 {
    println!("RUSTPI");
    clear_bss();
    board::init();
    println!("board init ok");
    static_check();
    mm::heap::init();
    println!("heap init ok");
    mm::page_pool::init();
    println!("page pool init ok");
    lib::address_space::init();
    println!("process pool init ok");
    lib::thread::init();
    println!("thread pool init ok");

    board::launch_other_cores();
    println!("launched other cores");
  }
  board::init_per_core();
  println!("init core {}", core_id);
  if core_id == 0 {
    // extern "C" {
    //   static KERNEL_ELF: [u8; 0x40000000];
    // }
    // init_backtrace(&KERNEL_ELF);
    // println!("init_backtrace ok");
    // init_backtrace_context();
    // println!("init_backtrace_context ok");
  }

  if core_id == 0 {
    // Note: `arg` is used to start different programs:
    //    0 - fktest: a `fork` test
    //    1 - pingpong: an IPC test
    //    2 - heap_test: test copy on write of heap
    #[cfg(target_arch = "aarch64")]
      lib::address_space::create(&lib::user_image::_binary_user_aarch64_elf_start, 3);
    #[cfg(target_arch = "riscv64")]
      lib::address_space::create(&lib::user_image::_binary_user_riscv64_elf_start, 3);
  }
  barrier();
  lib::scheduler::schedule();
  start_first_thread(core_id)
}

fn start_first_thread(core_id: CoreId) -> ! {
  extern {
    fn pop_context_first(ctx: usize, core_id: usize) -> !;
  }
  let t = current_thread().unwrap();
  let lock = t.context();
  let ctx_on_stack = *lock;
  drop(lock);
  unsafe {
    pop_context_first(&ctx_on_stack as *const _ as usize, core_id);
  }
}
