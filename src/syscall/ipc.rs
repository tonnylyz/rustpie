use super::{Result, SyscallOutRegisters::*};
use crate::lib::thread::{thread_sleep, thread_wake, Tid};
use common::syscall::error::*;
use crate::lib::traits::ContextFrameTrait;

#[inline(never)]
pub fn itc_receive() -> Result {
  let t = super::current_thread()?;
  t.ready_to_receive();
  if let Some(0) = t.is_serving() {
    t.ready_to_serve();
  }
  thread_sleep(&t);
  crate::lib::cpu::cpu().schedule();
  Ok(Unit)
}

#[inline(never)]
pub fn itc_send(tid: Tid, a: usize, b: usize, c: usize, d: usize) -> Result {
  let t = super::current_thread()?;
  let target = crate::lib::thread::thread_lookup(tid).ok_or_else(|| ERROR_INVARG)?;
  if target.receive() {
    target.map_with_context(|ctx| {
      ctx.set_syscall_result(&Result::Ok(Pentad(t.tid() as usize, a, b, c, d)));
    });
    thread_wake(&target);
    if let Some(caller) = t.is_serving() {
      if caller == target.tid() {
        t.ready_to_serve();
      }
    }
    Ok(Unit)
  } else {
    Err(ERROR_HOLD_ON)
  }
}

#[inline(never)]
pub fn itc_call(tid: Tid, a: usize, b: usize, c: usize, d: usize) -> Result {
  let t = super::current_thread()?;
  let target = crate::lib::thread::thread_lookup(tid).ok_or_else(|| ERROR_INVARG)?;

  if target.serve(t.tid()) {
    target.map_with_context(|ctx| {
      ctx.set_syscall_result(&Result::Ok(Pentad(t.tid() as usize, a, b, c, d)));
    });
    thread_wake(&target);
    t.ready_to_receive();
    thread_sleep(&t);
    crate::lib::cpu::cpu().schedule();
    Ok(Unit)
  } else {
    Err(ERROR_HOLD_ON)
  }
}
