use common::event::{EVENT_INTERRUPT, EVENT_THREAD_EXIT};
use common::syscall::error::ERROR_INVARG;
use spin::Once;

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
        thread_exit_sem().wait(t.clone());
        super::thread::thread_yield()
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

static THREAD_EXIT_SEM: Once<Semaphore> = Once::new();

fn thread_exit_sem() -> &'static Semaphore {
  if let Some(s) = THREAD_EXIT_SEM.get() {
    s
  } else {
    THREAD_EXIT_SEM.call_once(|| Semaphore::new())
  }
}

// called when a thread exits
pub fn thread_exit_signal(tid_exited: Tid) {
  match thread_exit_sem().try_signal() {
    None => {}
    Some(waiter) => {
      waiter.map_with_context(|ctx| {
        ctx.set_syscall_result(&SyscallResult::Ok(SyscallOutRegisters::Single(tid_exited as usize)));
      });
      thread_wake(&waiter);
    }
  }
}

