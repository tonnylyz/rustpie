use alloc::boxed::Box;

use libtrusted::wrapper::request_wrapper;
use microcall::message::Message;

#[inline(never)]
fn make_page_fault() {
  unsafe { (0xdeadbeef0000 as *mut usize).write(0); }
  panic!(); // indicates an exception may happen
}

struct ResourceA;

struct ResourceB;

impl Drop for ResourceA {
  fn drop(&mut self) {
    info!("resource a drop")
  }
}

#[inline(never)]
fn process(_msg: Message, _tid: usize) {
  info!("server called");
  let a = Box::new(ResourceA);
  // panic!();
  make_page_fault();
  let b = Box::new(ResourceB);
  Box::leak(a);
  Box::leak(b);
}

pub fn server() {
  microcall::server_register(common::server::SERVER_TEST).expect("server register failed");
  loop {
    let (client_tid, msg) = Message::receive().unwrap();
    request_wrapper(process, msg, client_tid).unwrap();
  }
}