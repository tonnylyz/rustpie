use common::syscall::error::ERROR_HOLD_ON;
use crate::sched_yield;

#[derive(Copy, Clone)]
pub struct Message(pub usize, pub usize, pub usize, pub usize);

pub fn call(server_id: usize, msg: Message) -> Result<Message, &'static str> {
  let server_tid = microcall::server_tid_wait(server_id);
  loop {
    match microcall::itc_call(server_tid, msg.0, msg.1, msg.2, msg.3) {
      Ok((_, a, b, c, d)) => {
        break Ok(Message(a, b, c, d))
      }
      Err(ERROR_HOLD_ON) => {
        sched_yield();
      }
      _ => {
        break Err("call failed")
      }
    }
  }
}
