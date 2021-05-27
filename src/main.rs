#![no_std]
#![no_main]
#![feature(global_asm)]
#![feature(llvm_asm)]
#![feature(alloc_error_handler)]
#![feature(panic_info_message)]
#![feature(format_args_nl)]
#![feature(lang_items)]
#![feature(const_btree_new)]
#![feature(const_generics)]
#![feature(const_evaluatable_checked)]

#[macro_use]
extern crate alloc;
extern crate rlibc;

#[macro_use]
mod misc;

#[macro_use]
extern crate log;

#[cfg(target_arch = "aarch64")]
#[path = "arch/aarch64/mod.rs"]
mod arch;

#[cfg(target_arch = "riscv64")]
#[path = "arch/riscv64/mod.rs"]
mod arch;

#[cfg(target_arch = "aarch64")]
#[path = "board/aarch64_virt.rs"]
mod board;

#[cfg(target_arch = "riscv64")]
#[path = "board/riscv64_virt.rs"]
mod board;

#[cfg(target_arch = "aarch64")]
#[path = "driver/aarch64_virt/mod.rs"]
mod driver;

#[cfg(target_arch = "riscv64")]
#[path = "driver/riscv64_virt/mod.rs"]
mod driver;

mod lib;
mod mm;
mod panic;
mod util;
mod backtracer;
mod logger;

use core::mem::size_of;
use arch::{CoreId, ContextFrame};
use lib::traits::*;
use lib::cpu::CoreTrait;
use lib::interrupt::InterruptController;
use mm::page_table::PageTableTrait;
use mm::page_table::PageTableEntryAttrTrait;

pub fn core_id() -> CoreId {
  crate::arch::Arch::core_id()
}

fn clear_bss() {
  extern "C" {
    fn BSS_START();
    fn BSS_END();
  }
  let start = (BSS_START as usize).pa2kva();
  let end = (BSS_END as usize).pa2kva();
  unsafe { core::ptr::write_bytes(start as *mut u8, 0, end - start); }
}

fn static_check() {
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
    clear_bss();
    board::init();
    static_check();
    mm::heap::init();
    logger::init();
    info!("heap init ok");
    mm::page_pool::init();
    info!("page pool init ok");
    lib::address_space::init();
    info!("process pool init ok");
    lib::thread::init();
    info!("thread pool init ok");

    board::launch_other_cores();
    info!("launched other cores");
  }
  board::init_per_core();

  crate::lib::cpu::current().create_idle_thread();
  info!("init core {}", core_id);
  if core_id == 0 {
    extern "C" {
      static KERNEL_ELF: [u8; 0x40000000];
    }
    panic::init(&KERNEL_ELF);
    info!("panic init ok");
  }

  if core_id == 0 {
    #[cfg(target_arch = "aarch64")]
      #[cfg(not(feature = "user_release"))]
      let bin = include_bytes!("../user/target/aarch64/debug/rustpi-user");

    #[cfg(target_arch = "aarch64")]
      #[cfg(feature = "user_release")]
      let bin = include_bytes!("../user/target/aarch64/release/rustpi-user");

    #[cfg(target_arch = "riscv64")]
      #[cfg(not(feature = "user_release"))]
      let bin = include_bytes!("../user/target/riscv64/debug/rustpi-user");

    #[cfg(target_arch = "riscv64")]
      #[cfg(feature = "user_release")]
      let bin = include_bytes!("../user/target/riscv64/release/rustpi-user");

    let (a, entry) = lib::address_space::load_image(bin);
    info!("load_image ok");
    // Note: `arg` is used to start different programs:
    //    0 - fktest: a `fork` test
    //    1 - virtio_blk server
    const INIT_ARG: usize = 3;

    let page_table = a.page_table();

    page_table.insert_page(common::CONFIG_USER_STACK_TOP - arch::PAGE_SIZE,
                           mm::UserFrame::new_memory(mm::page_pool::alloc()),
                           mm::page_table::EntryAttribute::user_default()).unwrap();
    info!("user stack ok");
    let t = crate::lib::thread::new_user(entry, common::CONFIG_USER_STACK_TOP, INIT_ARG, a.clone(), None);
    t.set_status(crate::lib::thread::Status::TsRunnable);

    for device in board::devices() {
      for uf in device.to_user_frames().iter() {
        a.page_table().insert_page(0x8_0000_0000 + uf.pa(), uf.clone(), mm::page_table::EntryAttribute::user_device()).unwrap();
      }
      for i in device.interrupts.iter() {
        crate::driver::INTERRUPT_CONTROLLER.enable(*i);
      }
    }
    info!("device added to user space");
  }

  util::barrier();
  crate::lib::cpu::current().schedule();

  extern {
    fn pop_context_first(ctx: usize, core_id: usize) -> !;
  }
  let t = crate::lib::cpu::current().running_thread().unwrap();
  let ctx = t.context();
  pop_context_first(&ctx as *const _ as usize, core_id);
}
