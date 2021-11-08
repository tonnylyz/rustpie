#![no_std]
#![no_main]
#![feature(global_asm)]
#![feature(asm)]
#![feature(alloc_error_handler)]
#![feature(panic_info_message)]
#![feature(format_args_nl)]
#![feature(lang_items)]
#![feature(const_btree_new)]
#![feature(allocator_api)]

#[macro_use]
extern crate alloc;
extern crate rlibc;

#[macro_use]
mod misc;

#[macro_use]
extern crate log;

#[macro_use]
extern crate static_assertions;

cfg_if::cfg_if! {
  if #[cfg(target_arch = "aarch64")] {
    #[path = "arch/aarch64/mod.rs"]
    mod arch;
    #[path = "board/aarch64_virt.rs"]
    mod board;
    #[path = "driver/aarch64_virt/mod.rs"]
    mod driver;
    // Note: size of ContextFrame needs to be synced with `arch/*/exception.S`
    assert_eq_size!([u8; 0x110], ContextFrame);
  } else if #[cfg(target_arch = "riscv64")] {
    #[path = "arch/riscv64/mod.rs"]
    mod arch;

    #[cfg(feature = "k210")]
    #[path = "board/riscv64_k210.rs"]
    mod board;

    #[cfg(not(feature = "k210"))]
    #[path = "board/riscv64_virt.rs"]
    mod board;

    #[path = "driver/riscv64_virt/mod.rs"]
    mod driver;
    assert_eq_size!([u8; 0x110], ContextFrame);
  } else {
    compile_error!("unsupported target_arch");
  }
}


mod lib;
mod mm;
mod panic;
mod util;
mod logger;
mod syscall;

use arch::ContextFrame;
use lib::traits::*;
use lib::interrupt::InterruptController;
use mm::page_table::PageTableTrait;
use mm::page_table::PageTableEntryAttrTrait;

#[macro_use]
mod macros {
  #[repr(C)] // guarantee 'bytes' comes after '_align'
  pub struct AlignedAs<Align, Bytes: ?Sized> {
    pub _align: [Align; 0],
    pub bytes: Bytes,
  }

  macro_rules! include_bytes_align_as {
  ($align_ty:ty, $path:literal) => {
    {  // const block expression to encapsulate the static
      use $crate::macros::AlignedAs;

      // this assignment is made possible by CoerceUnsized
      static ALIGNED: &AlignedAs::<$align_ty, [u8]> = &AlignedAs {
        _align: [],
        bytes: *include_bytes!($path),
      };

      &ALIGNED.bytes
    }
  };
}
}

#[repr(align(4096))]
struct AlignPage;

#[no_mangle]
pub unsafe fn main(core_id: arch::CoreId) -> ! {
  crate::arch::Arch::exception_init();
  if core_id == 0 {
    board::init();
    mm::heap::init();
    let _ = logger::init();
    info!("heap init ok");
    mm::page_pool::init();
    info!("page pool init ok");

    board::launch_other_cores();
    info!("launched other cores");
  }
  board::init_per_core();
  info!("init core {}", core_id);

  if core_id == 0 {
    #[cfg(target_arch = "aarch64")]
      #[cfg(not(feature = "user_release"))]
      let bin = include_bytes_align_as!(AlignPage, "../trusted/target/aarch64/debug/trusted");

    #[cfg(target_arch = "aarch64")]
      #[cfg(feature = "user_release")]
      let bin = include_bytes_align_as!(AlignPage, "../trusted/target/aarch64/release/trusted");

    #[cfg(target_arch = "riscv64")]
      #[cfg(not(feature = "user_release"))]
      let bin = include_bytes_align_as!(AlignPage, "../trusted/target/riscv64/debug/trusted");

    #[cfg(target_arch = "riscv64")]
      #[cfg(feature = "user_release")]
      let bin = include_bytes_align_as!(AlignPage, "../trusted/target/riscv64/release/trusted");

    info!("embedded trusted {:x}", bin.as_ptr() as usize);
    let (a, entry) = lib::address_space::load_image(bin);
    info!("load_image ok");

    let page_table = a.page_table();
    let stack_frame = mm::page_pool::page_alloc().expect("failed to allocate trusted stack");
    page_table.insert_page(common::CONFIG_USER_STACK_TOP - arch::PAGE_SIZE,
                           mm::Frame::from(stack_frame),
                           mm::page_table::EntryAttribute::user_default()).unwrap();


    let dma_frame = mm::page_pool::page_alloc().expect("failed to allocate trusted dma frame");
    let dma_frame_no_cache = dma_frame.pa() - 0x40000000;
    page_table.insert_page(0x8_0000_0000,
                           mm::Frame::Device(dma_frame_no_cache),
                           mm::page_table::EntryAttribute::user_device()).unwrap();

    info!("user stack ok");
    let t = crate::lib::thread::new_user(
      entry,
      common::CONFIG_USER_STACK_TOP,
      0,
      a.clone(),
      None,
    );
    lib::thread::thread_wake(&t);

    for device in board::devices() {
      for uf in device.to_user_frames().iter() {
        a.page_table().insert_page(
          0x8_0000_0000 + uf.pa(),
          uf.clone(),
          mm::page_table::EntryAttribute::user_device(),
        ).unwrap();
      }
      for i in device.interrupts.iter() {
        crate::driver::INTERRUPT_CONTROLLER.enable(*i);
      }
    }
    info!("device added to user space");
  }

  util::barrier();
  lib::cpu::cpu().schedule();

  extern {
    fn pop_context_first(ctx: usize, core_id: usize) -> !;
  }
  match lib::cpu::cpu().running_thread() {
    None => panic!("no running thread"),
    Some(t) => {
      let ctx = t.context();
      pop_context_first(&ctx as *const _ as usize, core_id);
    }
  }
}
