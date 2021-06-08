use microcall::{itc_send, itc_receive};

#[repr(C)]
#[derive(Copy, Clone, Debug, Default)]
pub struct Message {
  pub a: usize,
  pub b: usize,
  pub c: usize,
  pub d: usize,
}

impl Message {
  pub fn receive() -> (u16, Self) {
    let mut msg = Message::default();
    let sender = itc_receive(&mut msg as *mut _ as usize) as u16;
    (sender, msg)
  }
  pub fn send_to(&self, tid: u16) -> isize {
    itc_send(tid, self.a, self.b, self.c, self.d)
  }
}
