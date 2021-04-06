#![no_std]
#![no_main]
#![feature(global_asm)]
#![feature(llvm_asm)]
#![feature(alloc_error_handler)]
#![feature(panic_info_message)]
#![feature(format_args_nl)]
#![feature(lang_items)]
#![feature(const_btree_new)]

#[macro_use]
extern crate alloc;
extern crate rlibc;
// extern crate fatfs;
// extern crate core_io;

use crate::arch::ArchTrait;
pub use crate::arch::CoreId;
use crate::lib::core::CoreTrait;
use crate::lib::interrupt::InterruptController;

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
  crate::arch::Arch::core_id()
}

fn clear_bss() {
  use arch::Address;
  use rlibc::memset;
  extern "C" {
    fn BSS_START();
    fn BSS_END();
  }
  let start = (BSS_START as usize).pa2kva();
  let end = (BSS_END as usize).pa2kva();
  unsafe { memset(start as *mut u8, 0, end - start); }
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
  crate::arch::Arch::exception_init();
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
    // extern "C" {
    //   static KERNEL_ELF: [u8; 0x40000000];
    // }
    // panic::init_backtrace(&KERNEL_ELF);
    // println!("init_backtrace ok");
    // panic::init_backtrace_context();
    // println!("init_backtrace_context ok");
  }

  if core_id == 0 {
    // Note: `arg` is used to start different programs:
    //    0 - fktest: a `fork` test
    //    1 - pingpong: an IPC test
    //    2 - heap_test: test copy on write of heap
    //    3 - virtio_blk server
    #[cfg(target_arch = "aarch64")]
      let (a, entry) = lib::address_space::load_image(&lib::user_image::_binary_user_aarch64_elf_start);
    #[cfg(target_arch = "riscv64")]
      let (a, entry) = lib::address_space::load_image(&lib::user_image::_binary_user_riscv64_elf_start);

    let page_table = a.page_table();

    use lib::page_table::PageTableTrait;
    use lib::page_table::PageTableEntryAttrTrait;
    page_table.insert_page(config::CONFIG_USER_STACK_TOP - arch::PAGE_SIZE,
                           mm::UserFrame::new_memory(mm::page_pool::alloc()),
                           lib::page_table::EntryAttribute::user_default()).unwrap();
    let t = crate::lib::thread::new_user(entry, config::CONFIG_USER_STACK_TOP, 3, a.clone(), None);
    t.set_status(crate::lib::thread::Status::TsRunnable);


    use crate::lib::device::*;
    let virtio_mmio = Device::new(vec![
      0x0a000000..0x0a000200
    ], vec![
      0x10 + 32
    ]);
    for uf in virtio_mmio.to_user_frames().iter() {
      a.page_table().insert_page(0x8_0000_0000 + uf.pa(), uf.clone(), lib::page_table::EntryAttribute::user_device());
    }
    for i in virtio_mmio.interrupts.iter() {
      crate::driver::INTERRUPT_CONTROLLER.enable(*i);
    }
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
