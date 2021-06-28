#![no_std]
#![no_main]
#![feature(format_args_nl)]
#![feature(const_btree_new)]

#[macro_use]
extern crate alloc;
extern crate rlibc;

#[macro_use]
extern crate log;

#[macro_use]
extern crate libtrusted;

mod blk;
mod fs;
mod root;
mod terminal;
mod mm;
mod pm;
mod logger;

#[no_mangle]
fn _start(_arg: usize) -> ! {
  libtrusted::mm::heap_init();
  logger::init();
  info!("trusted root start");
  root::main();
  microcall::thread_destroy(0);
  loop {};
}
