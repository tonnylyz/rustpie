#![no_std]
#![no_main]
#![feature(global_asm)]
#![feature(alloc_error_handler)]
#![feature(panic_info_message)]
#![feature(core_intrinsics)]
#![feature(format_args_nl)]
#![feature(llvm_asm)]
#![feature(lang_items)]
#![feature(array_map)]

#[macro_use]
extern crate alloc;
extern crate rlibc;
extern crate fatfs;
extern crate core_io;

pub use crate::arch::CoreId;
use crate::lib::core::CoreTrait;

#[macro_use]
mod misc;

mod arch;
mod board;
mod driver;
mod lib;
mod mm;
mod config;
mod panic;

pub fn core_id() -> CoreId {
  use crate::arch::ArchTrait;
  crate::arch::Arch::core_id()
}

fn clear_bss() {
  use arch::Address;
  extern "C" {
    fn BSS_START();
    fn BSS_END();
  }
  let start = (BSS_START as usize).pa2kva();
  let end = (BSS_END as usize).pa2kva();
  unsafe { core::intrinsics::volatile_set_memory(start as *mut u8, 0, end - start); }
}

fn static_check() {
  use arch::ContextFrame;
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
pub unsafe fn main(core_id: arch::CoreId) -> ! {
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

  crate::lib::core::current().create_idle_thread();
  println!("init core {}", core_id);
  if core_id == 0 {
    // lib::fatfs::init();
    extern "C" {
      static KERNEL_ELF: [u8; 0x40000000];
    }
    panic::init_backtrace(&KERNEL_ELF);
    println!("init_backtrace ok");
    panic::init_backtrace_context();
    println!("init_backtrace_context ok");
  }

  if core_id == 0 {
    // Note: `arg` is used to start different programs:
    //    0 - fktest: a `fork` test
    //    1 - pingpong: an IPC test
    //    2 - heap_test: test copy on write of heap
    #[cfg(target_arch = "aarch64")]
      let (a, entry) = lib::address_space::load_image(&lib::user_image::_binary_user_aarch64_elf_start);
    #[cfg(target_arch = "riscv64")]
      let (a, entry) = lib::address_space::load_image(&lib::user_image::_binary_user_riscv64_elf_start);

    let page_table = a.page_table();

    use lib::page_table::PageTableTrait;
    use lib::page_table::PageTableEntryAttrTrait;
    match page_table.insert_page(config::CONFIG_USER_STACK_TOP - arch::PAGE_SIZE,
                                 mm::page_pool::alloc(),
                                 lib::page_table::EntryAttribute::user_default()) {
      Ok(_) => {}
      Err(_) => { panic!("process: create: page_table.insert_page failed") }
    }
    let t = crate::lib::thread::new_user(entry, config::CONFIG_USER_STACK_TOP, 0, a.clone(), None);
    t.set_status(crate::lib::thread::Status::TsRunnable);
  }

  lib::barrier::barrier();
  crate::lib::core::current().schedule();

  extern {
    fn pop_context_first(ctx: usize, core_id: usize) -> !;
  }
  let t = crate::lib::core::current().running_thread().unwrap();
  let ctx = t.context();
  pop_context_first(&ctx as *const _ as usize, core_id);
}
