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

use rpabi::platform::PlatformInfo;
#[cfg(feature = "error_unwind")]
use unwind::catch::catch_unwind;

#[macro_use]
mod common;

mod blk;
mod serial;

mod fs;
mod logger;
mod mm;
mod pm;
mod root;
mod rtc;

#[no_mangle]
extern "C" fn _start(arg: *const PlatformInfo) -> ! {
  rpsyscall::set_exception_handler(common::exception::handler as usize)
    .expect("set exception handler failed");
  common::mm::heap_init();
  logger::init().expect("logger init failed");
  info!("trusted root start");
  #[cfg(feature = "error_unwind")]
  {
    let r = catch_unwind(|| {
      root::main(unsafe { arg.as_ref().unwrap() });
    });
    match r {
      Ok(_) => {}
      Err(_) => error!("root died"),
    }
  }
  #[cfg(not(feature = "error_unwind"))]
  root::main(unsafe { arg.as_ref().unwrap() });
  let _ = rpsyscall::thread_destroy(0);
  loop {}
}
