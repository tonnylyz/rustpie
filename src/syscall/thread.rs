use super::{Result, SyscallOutRegisters::*};
use crate::lib::thread::{Tid, thread_wake, thread_sleep};
use common::syscall::error::{ERROR_DENIED, ERROR_INVARG};

#[inline(never)]
#[inject::count_stmts]
pub fn get_tid() -> Result {
  Ok(Single(super::current_thread()?.tid()))
}

#[inline(never)]
#[inject::count_stmts]
pub fn thread_yield() -> Result {
  // let icntr = crate::lib::timer::current_cycle();
  crate::lib::cpu::cpu().schedule();
  // let icntr2 = crate::lib::timer::current_cycle();
  // info!("as create cycle {}", icntr2 - icntr);
  Ok(Unit)
}

#[inline(never)]
#[inject::count_stmts]
pub fn thread_destroy(tid: Tid) -> Result {
  let current_thread = super::current_thread()?;
  if tid == 0 {
    crate::lib::thread::thread_destroy(current_thread);
    thread_yield()
  } else {
    match crate::lib::thread::thread_lookup(tid) {
      None => Err(ERROR_INVARG),
      Some(t) => {
        if t.is_child_of(current_thread.tid()) {
          crate::lib::thread::thread_destroy(t);
          Ok(Unit)
        } else {
          Err(ERROR_DENIED)
        }
      }
    }
  }
}

#[inline(never)]
#[inject::count_stmts]
pub fn thread_alloc(asid: u16, entry: usize, sp: usize, arg: usize) -> Result {
  let t = super::current_thread()?;
  let a = super::lookup_as(asid)?;
  let child_thread = crate::lib::thread::new_user(entry, sp, arg, a.clone(), Some(t.tid()));
  Ok(Single(child_thread.tid() as usize))
}

#[inline(never)]
#[inject::count_stmts]
pub fn thread_set_status(tid: usize, status: usize) -> Result {
  use common::thread::*;
  let runnable = match status {
    THREAD_STATUS_NOT_RUNNABLE => false,
    THREAD_STATUS_RUNNABLE => true,
    _ => return Err(ERROR_INVARG)
  };
  match crate::lib::thread::thread_lookup(tid) {
    None => Err(ERROR_INVARG),
    Some(t) => {
      if runnable {
        thread_wake(&t);
      } else {
        thread_sleep(&t, crate::lib::thread::Status::Sleep);
      }
      Ok(Unit)
    }
  }
}
