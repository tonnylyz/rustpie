use alloc::boxed::Box;

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

// #[inline(never)]
// fn make_page_fault() {
//   static mut HAPPENED: bool = false;
//   unsafe {
//     if !HAPPENED {
//       HAPPENED = true;
//       (0xdeadbeef0000 as *mut usize).write(0);
//       panic!(); // indicates an exception may happen
//     }
//   }
// }
//
// struct ResourceA;
//
// struct ResourceB;
//
// impl Drop for ResourceA {
//   fn drop(&mut self) {
//     info!("resource a drop")
//   }
// }
//
// #[inline(never)]
// fn process(_msg: Message, _tid: usize) {}

pub fn server() {
  info!("test server t{}", microcall::get_tid());
  microcall::server_register(common::server::SERVER_TEST).expect("server register failed");
  let (mut client_tid, mut msg) = Message::receive().unwrap();
  loop {
    let end = current_cycle();
    let mut result = Message::default();
    result.a = end;
    let next = result.reply_recv(client_tid).unwrap();
    client_tid = next.0;
    msg = next.1;
  }
}
