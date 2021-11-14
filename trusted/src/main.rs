#![no_std]
#![no_main]
#![feature(format_args_nl)]
#![feature(const_btree_new)]
#![feature(allocator_api)]
#![feature(asm)]

#[macro_use]
extern crate alloc;
#[macro_use]
extern crate libtrusted;
#[macro_use]
extern crate log;
extern crate rlibc;

use unwind::catch::catch_unwind;

mod blk;
mod fs;
mod root;
mod terminal;
mod mm;
mod pm;
mod logger;
mod test;
mod rtc;
mod panic;

#[no_mangle]
fn _start(_arg: usize) -> ! {
  microcall::set_exception_handler(libtrusted::exception::handler as usize).expect("set exception handler failed");
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
  let _ = microcall::thread_destroy(0);
  loop {};
}
