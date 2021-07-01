use common::event::*;
use spin::Mutex;

use crate::lib::thread::Thread;

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

static THREAD_EXIT_WAITER: Mutex<Option<Thread>> = Mutex::new(None);

pub fn set_thread_exit_waiter(thread: Thread) {
  let mut waiter = THREAD_EXIT_WAITER.lock();
  *waiter = Some(thread.clone());
}

pub fn thread_exit_waiter() -> Option<Thread> {
  let waiter = THREAD_EXIT_WAITER.lock();
  waiter.clone()
}
