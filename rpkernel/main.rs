#![no_std]
#![no_main]
#![feature(panic_info_message)]
// for ::try_new allocation
#![feature(allocator_api)]
// for unwind feature: eh_personality and so on
#![allow(internal_features)]
#![feature(lang_items)]
#![feature(abi_x86_interrupt)]

#[macro_use]
extern crate alloc;
#[macro_use]
extern crate log;
#[macro_use]
extern crate static_assertions;

use arch::ContextFrame;
use kernel::interrupt::InterruptController;
use kernel::traits::*;

pub const MAX_CPU_NUMBER: usize = 4;
pub use board::core_id;
pub use board::cpu_number;
use rpabi::syscall::mm::EntryAttribute;

#[macro_use]
mod misc;

cfg_if::cfg_if! {
  if #[cfg(target_arch = "aarch64")] {
    #[path = "arch/aarch64/mod.rs"]
    mod arch;

    #[cfg(feature = "virt")]
    #[path = "board/aarch64_virt.rs"]
    mod board;

    #[path = "driver/aarch64/mod.rs"]
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

    #[path = "driver/riscv64/mod.rs"]
    mod driver;
    assert_eq_size!([u8; 0x110], ContextFrame);
  } else if #[cfg(target_arch = "x86_64")] {
    
    #[path = "arch/x86_64/mod.rs"]
    mod arch;

    #[path = "board/x86_64_virt.rs"]
    mod board;

    #[path = "driver/x86_64/mod.rs"]
    mod driver;
    assert_eq_size!([u8; 0xA0], ContextFrame);

  } else {
    compile_error!("unsupported target_arch");
  }
}

mod kernel;
mod logger;
mod mm;
mod panic;
mod syscall;
mod util;

#[macro_use]
mod macros {
  #[repr(C)] // guarantee 'bytes' comes after '_align'
  pub struct AlignedAs<Align, Bytes: ?Sized> {
    pub _align: [Align; 0],
    pub bytes: Bytes,
  }

  macro_rules! include_bytes_align_as {
    ($align_ty:ty, $path:literal) => {{
      // const block expression to encapsulate the static
      use $crate::macros::AlignedAs;

      // this assignment is made possible by CoerceUnsized
      static ALIGNED: &AlignedAs<$align_ty, [u8]> = &AlignedAs {
        _align: [],
        bytes: *include_bytes!($path),
      };

      &ALIGNED.bytes
    }};
  }
}

#[repr(align(4096))]
struct AlignPage;

#[no_mangle]
extern "C" fn main(core_id: arch::CoreId, boot_data: usize) -> ! {
  crate::arch::Arch::exception_init();
  if core_id == 0 {
    let (hr, pr) = board::init(boot_data);
    println!("Heap {:x?} Paged {:x?}", hr, pr);

    mm::heap::init(hr);
    let _ = logger::init();
    info!("heap init ok");

    mm::page_pool::init(pr);
    info!("page pool init ok");

    board::launch_other_cores();
    info!("launched other cores");
  }
  board::init_per_core();
  info!("init core {}", core_id);

  util::barrier();
  if core_id == 0 {
    #[cfg(target_arch = "aarch64")]
    #[cfg(not(feature = "user_release"))]
    let bin = include_bytes_align_as!(AlignPage, "../trusted/target/aarch64-unknown-rustpi/debug/trusted.bin");

    #[cfg(target_arch = "aarch64")]
    #[cfg(feature = "user_release")]
    let bin = include_bytes_align_as!(AlignPage, "../trusted/target/aarch64-unknown-rustpi/release/trusted.bin");

    #[cfg(target_arch = "riscv64")]
    #[cfg(not(feature = "user_release"))]
    let bin = include_bytes_align_as!(AlignPage, "../trusted/target/riscv64gc-unknown-rustpi-elf/debug/trusted.bin");

    #[cfg(target_arch = "riscv64")]
    #[cfg(feature = "user_release")]
    let bin = include_bytes_align_as!(AlignPage, "../trusted/target/riscv64gc-unknown-rustpi-elf/release/trusted.bin");

    #[cfg(target_arch = "x86_64")]
    #[cfg(not(feature = "user_release"))]
    let bin = include_bytes_align_as!(AlignPage, "../trusted/target/x86_64-unknown-rustpi/debug/trusted.bin");

    #[cfg(target_arch = "x86_64")]
    #[cfg(feature = "user_release")]
    let bin = include_bytes_align_as!(AlignPage, "../trusted/target/x86_64-unknown-rustpi/release/trusted.bin");

    info!("embedded trusted {:x}", bin.as_ptr() as usize);
    let (a, entry) = kernel::address_space::load_image(bin);
    info!("load_image ok");

    let mut page_table = a.page_table();
    let stack_frame = mm::page_pool::page_alloc().expect("failed to allocate trusted stack");
    page_table
      .insert_page(
        rpabi::CONFIG_USER_STACK_TOP - arch::PAGE_SIZE,
        mm::Frame::from(stack_frame),
        EntryAttribute::user_default(),
      )
      .unwrap();
    let plat_info_pa = (&board::PLATFORM_INFO as *const _ as usize).kva2pa();
    page_table
      .insert_page(
        rpabi::CONFIG_TRUSTED_PLATFORM_INFO,
        mm::Frame::from(mm::PhysicalFrame::new(plat_info_pa)),
        EntryAttribute::user_readonly(),
      )
      .unwrap();

    #[cfg(feature = "k210")]
    {
      let dma_frame = mm::page_pool::page_alloc().expect("failed to allocate trusted dma frame");
      let dma_frame_no_cache = dma_frame.pa() - 0x40000000;
      info!("dma_frame {:016x}", dma_frame_no_cache);
      page_table
        .insert_page(
          rpabi::platform::USER_SPACE_DRIVER_MMIO_OFFSET,
          mm::Frame::Device(dma_frame_no_cache),
          EntryAttribute::user_device(),
        )
        .unwrap();
      core::mem::forget(dma_frame);
    }
    info!("user stack ok");
    let t = crate::kernel::thread::new_user(
      entry,
      rpabi::CONFIG_USER_STACK_TOP,
      rpabi::CONFIG_TRUSTED_PLATFORM_INFO,
      a.clone(),
      None,
    );
    kernel::thread::thread_wake(&t);

    for device in &board::PLATFORM_INFO.get().unwrap().devices {
      if let Some(device) = device {
        for uf in kernel::device::device_to_user_frames(device).iter() {
          page_table
            .insert_page(
              rpabi::platform::USER_SPACE_DRIVER_MMIO_OFFSET + uf.pa(),
              uf.clone(),
              rpabi::syscall::mm::EntryAttribute::user_device(),
            )
            .unwrap();
        }
        for i in device.interrupt.iter() {
          crate::driver::INTERRUPT_CONTROLLER.enable(*i);
        }
      }
    }
    info!("device added to user space");
    drop(page_table);
  }

  util::barrier();
  kernel::cpu::cpu().tick(false);

  extern "C" {
    fn pop_context_first(ctx: usize, core_id: usize) -> !;
  }
  match kernel::cpu::cpu().running_thread() {
    None => panic!("no running thread"),
    Some(t) => {
      let ctx = t.context();
      unsafe { pop_context_first(&ctx as *const _ as usize, core_id) };
    }
  }
}
