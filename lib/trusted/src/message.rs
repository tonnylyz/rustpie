use microcall::{itc_send, itc_receive};

pub type Error = usize;

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
    match itc_receive() {
      Ok((sender, a, b, c, d)) => {
        (sender, Message {
          a, b, c, d
        })
      }
      Err(_) => panic!("message receive error")
    }
  }
  pub fn send_to(&self, tid: u16) -> Result<(), Error> {
    itc_send(tid, self.a, self.b, self.c, self.d)
  }
}
