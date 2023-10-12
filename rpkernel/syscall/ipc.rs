use rpabi::syscall::error::*;

use crate::kernel::thread::{thread_sleep, Tid};
use crate::kernel::thread::Status as ThreadStatus;
use crate::kernel::traits::ContextFrameTrait;

use super::{Result, SyscallOutRegisters::*, VOID_SCHEDULE, VOID};

#[inline(never)]
pub fn itc_receive() -> Result {
  let t = super::current_thread()?;
  thread_sleep(&t, ThreadStatus::WaitForRequest);
  VOID_SCHEDULE
}

#[inline(never)]
pub fn itc_send(tid: Tid, a: usize, b: usize, c: usize, d: usize) -> Result {
  let current = super::current_thread()?;
  let target = crate::kernel::thread::thread_lookup(tid).ok_or_else(|| ERROR_INVARG)?;
  if target.wait_for_reply(|| {
    target.map_with_context(|ctx| {
      ctx.set_syscall_result(&Pentad(current.tid() as usize, a, b, c, d), 0);
    });
  }) {
    VOID
  } else {
    Err(ERROR_DENIED)
  }
}

#[inline(never)]
pub fn itc_call(tid: Tid, a: usize, b: usize, c: usize, d: usize) -> Result {
  let current = super::current_thread()?;
  let target = crate::kernel::thread::thread_lookup(tid).ok_or_else(|| ERROR_INVARG)?;
  if target.wait_for_request(|| {
    target.map_with_context(|ctx| {
      ctx.set_syscall_result(&Pentad(current.tid() as usize, a, b, c, d), 0);
    });
    thread_sleep(&current, crate::kernel::thread::Status::WaitForReply);
  }) {
    VOID_SCHEDULE
  } else {
    Err(ERROR_HOLD_ON)
  }
}

#[inline(never)]
pub fn itc_reply_recv(tid: Tid, a: usize, b: usize, c: usize, d: usize) -> Result {
  let current = super::current_thread()?;
  let target = crate::kernel::thread::thread_lookup(tid).ok_or_else(|| ERROR_INVARG)?;
  if !target.wait_for_reply(|| {
    target.map_with_context(|ctx| {
      ctx.set_syscall_result(&Pentad(current.tid() as usize, a, b, c, d), 0);
    });
  }) {
    warn!("t{} not wait for reply from t{} status {:?}", target.tid(), current.tid(), target.status());
  }
  thread_sleep(&current, ThreadStatus::WaitForRequest);
  VOID_SCHEDULE
}
