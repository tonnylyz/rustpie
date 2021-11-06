use alloc::vec::Vec;
use common::event::{EVENT_INTERRUPT, EVENT_THREAD_EXIT};
use common::syscall::error::{ERROR_HOLD_ON, ERROR_INVARG};
use spin::{Mutex, Once};

use crate::lib::interrupt::INT_SEM;
use crate::lib::semaphore::{Semaphore, SemaphoreWaitResult};
use crate::lib::thread::{thread_wake, Tid};
use crate::lib::traits::ContextFrameTrait;
use crate::syscall::{Result as SyscallResult, SyscallOutRegisters};

use super::{Result, SyscallOutRegisters::*};

#[inline(never)]
#[inject::count_stmts]
pub fn event_wait(event_type: usize, event_num: usize) -> Result {
  let t = super::current_thread()?;
  if let Some(e) = Event::from(event_type, event_num) {
    match e {
      Event::Interrupt(i) => {
        match INT_SEM.wait(t.clone(), i) {
          SemaphoreWaitResult::Acquired => Ok(Unit),
          SemaphoreWaitResult::Enqueued => super::thread::thread_yield(),
        }
      }
      Event::ThreadExit => {
        let mut q = TID_EXIT.lock();
        match q.pop() {
          None => Err(ERROR_HOLD_ON),
          Some(t) => Ok(Single(t as usize)),
        }
      }
    }
  } else {
    Err(ERROR_INVARG)
  }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Event {
  Interrupt(usize),
  ThreadExit,
}

impl Event {
  pub fn from(event_type: usize, event_num: usize) -> Option<Self> {
    match event_type {
      EVENT_INTERRUPT => Some(Event::Interrupt(event_num)),
      EVENT_THREAD_EXIT => Some(Event::ThreadExit),
      _ => None,
    }
  }
}

static TID_EXIT: Mutex<Vec<Tid>> = Mutex::new(Vec::new());

// called when a thread exits
pub fn thread_exit_signal(tid_exited: Tid) {
  let mut q = TID_EXIT.lock();
  q.push(tid_exited);
}

