#![no_std]
#![no_main]
#![feature(panic_info_message)]

// for unwind feature: eh_personality and so on
#![allow(internal_features)]
#![feature(lang_items)]

#[macro_use]
extern crate alloc;
#[macro_use]
extern crate log;


use unwind::catch::catch_unwind;

#[cfg(any(feature = "shyper", feature = "virt"))]
#[path = "blk/virtio_blk.rs"]
mod blk;

#[cfg(feature = "k210")]
#[path = "blk/k210_sdcard.rs"]
mod blk;

#[cfg(feature = "tx2")]
#[path = "blk/ramdisk.rs"]
mod blk;

#[macro_use]
mod libtrusted;

mod fs;
mod root;
mod terminal;
mod mm;
mod pm;
mod logger;
mod rtc;

#[no_mangle]
fn _start(_arg: usize) -> ! {
  rpsyscall::set_exception_handler(libtrusted::exception::handler as usize).expect("set exception handler failed");
  libtrusted::mm::heap_init();
  logger::init().expect("logger init failed");
  info!("trusted root start");
  let r = catch_unwind(|| {
    root::main();
  });
  match r {
    Ok(_) => {}
    Err(_) => error!("root died")
  }
  let _ = rpsyscall::thread_destroy(0);
  loop {};
}
