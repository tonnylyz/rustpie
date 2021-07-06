use common::event::*;
use spin::Once;

use crate::lib::semaphore::Semaphore;
use crate::lib::syscall::{SyscallOutRegisters, SyscallResult};
use crate::lib::thread::Tid;
use crate::lib::traits::ContextFrameTrait;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Event {
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

pub fn thread_exit_sem() -> &'static Semaphore {
  if let Some(s) = THREAD_EXIT_SEM.get() {
    s
  } else {
    THREAD_EXIT_SEM.call_once(|| Semaphore::new())
  }
}

pub fn thread_exit_signal(tid_exited: Tid) {
  match thread_exit_sem().try_signal() {
    None => {}
    Some(waiter) => {
      waiter.map_with_context(|ctx| {
        ctx.set_syscall_result(&SyscallResult::Ok(SyscallOutRegisters::Single(tid_exited as usize)));
      });
      waiter.wake();
    }
  }
}

