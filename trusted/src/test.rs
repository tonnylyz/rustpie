use alloc::boxed::Box;
use libtrusted::wrapper::request_wrapper;

// use libtrusted::wrapper::request_wrapper;
use microcall::message::Message;

#[allow(dead_code)]
#[cfg(target_arch = "aarch64")]
pub fn current_cycle() -> usize {
  let r;
  unsafe {
    asm!("mrs {}, pmccntr_el0", out(reg) r);
  }
  r
}

#[allow(dead_code)]
#[cfg(target_arch = "riscv64")]
pub fn current_cycle() -> usize {
  let r;
  unsafe {
    asm!("rdcycle {}", out(reg) r);
  }
  r
}

#[allow(dead_code)]
#[inline(never)]
fn make_page_fault() {
  unsafe { (0xdeadbeef0000 as *mut usize).write(0); }
  panic!();
}

#[allow(dead_code)]
struct ResourceA;

#[allow(dead_code)]
struct ResourceB;

impl Drop for ResourceA {
  fn drop(&mut self) {
    info!("resource a drop")
  }
}

#[inline(never)]
fn test(msg: Message, tid: usize) {
  let a = Box::new(ResourceA);
  if msg.a == 1 {
    info!("null called - server panic");
    panic!();
  } else {
    info!("null called - server page fault");
    make_page_fault();
  }
  let b = Box::new(ResourceB);
  Box::leak(a);
  Box::leak(b);
}

pub fn server() {
  info!("test server t{}", microcall::get_tid());
  microcall::server_register(common::server::SERVER_TEST).expect("server register failed");
  let (mut client_tid, mut msg) = Message::receive().unwrap();
  loop {
    let mut result = Message::default();
    if msg.a == 1 || msg.a == 2 {
      request_wrapper(test, msg, client_tid);
    } else {
      let end = current_cycle();
      result.a = end;
    }
    let next = result.reply_recv(client_tid).unwrap();
    client_tid = next.0;
    msg = next.1;
  }
}
