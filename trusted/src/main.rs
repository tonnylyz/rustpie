#![no_std]
#![no_main]
#![feature(format_args_nl)]
#![feature(const_btree_new)]
#![feature(allocator_api)]
#![feature(try_reserve)]
#![feature(asm)]

#[macro_use]
extern crate alloc;
extern crate rlibc;

#[macro_use]
extern crate log;

#[macro_use]
extern crate libtrusted;

use unwind::catch::catch_unwind;


mod blk;
mod fs;
mod root;
mod terminal;
mod mm;
mod pm;
mod logger;
mod test;

#[no_mangle]
fn _start(_arg: usize) -> ! {
  microcall::set_exception_handler(libtrusted::exception::handler as usize);
  libtrusted::mm::heap_init();
  logger::init();
  info!("trusted root start");
  let r = catch_unwind(|| {
    root::main();
  });
  match r {
    Ok(_) => {}
    Err(_) => error!("root died")
  }
  microcall::thread_destroy(0);
  loop {};
}
