use crate::syscall::*;
use alloc::boxed::Box;
use crate::config::PAGE_SIZE;

#[repr(C)]
#[derive(Copy, Clone, Debug, Default)]
pub struct ItcMessage {
  pub a: usize,
  pub b: usize,
  pub c: usize,
  pub d: usize,
}

impl ItcMessage {
  pub fn receive() -> (u16, Self) {
    let mut msg = ItcMessage::default();
    let sender = itc_receive(&mut msg as *mut _ as usize) as u16;
    println!("\t\t***t{} --> t{}", sender, get_tid());
    (sender, msg)
  }
  pub fn send_to(&self, tid: u16) -> isize
  {
    println!("\t\t***t{} --> t{}", get_tid(), tid);
    itc_send(tid, self.a, self.b, self.c, self.d)
  }
}

fn itc_test2(arg: usize) {
  println!("itc_test2: arg {}", arg);
  let mut msg = Box::new(ItcMessage {
    a: 0x1010101010,
    b: 0x2020202020,
    c: 0x3030303030,
    d: 0x4040404040,
  });
  itc_receive(msg.as_mut() as *mut _ as usize);
  println!("receive {:x?}", msg);
  loop {}
}

pub fn test() {
  use crate::arch::page_table::*;
  println!("itc_test start");
  mem_alloc(0, 0x1000_0000, PTE_DEFAULT);
  let t2 = thread_alloc(itc_test2 as usize, 0x1000_0000 + PAGE_SIZE, 0);
  thread_yield();
  for _ in 0..0x100000 {
    unsafe { llvm_asm!("nop"); }
  }
  let r = itc_send(t2, 0x11121314, 0x21222324, 0x31323334, 0x41424344);
  if r == 0 {
    println!("send ok");
  } else {
    println!("send error {}", r);
  }
  loop {}
}
