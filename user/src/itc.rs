use crate::syscall::*;

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
    println!("[ITC] R t{} -> t{}", sender, get_tid());
    (sender, msg)
  }
  pub fn send_to(&self, tid: u16) -> isize
  {
    println!("[ITC] T t{} -> t{}", get_tid(), tid);
    itc_send(tid, self.a, self.b, self.c, self.d)
  }
}
