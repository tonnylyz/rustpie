use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use rpabi::event::*;
use rpabi::syscall::error::{ERROR_HOLD_ON, ERROR_INVARG};
use spin::Mutex;

use crate::kernel::interrupt::INT_SEM;
use crate::kernel::semaphore::SemaphoreWaitResult;
use crate::kernel::thread::{Tid, thread_sleep};

use super::{Result, VOID, VOID_SCHEDULE};

#[inline(never)]
pub fn event_wait(event_type: usize, event_num: usize) -> Result {
  let t = super::current_thread()?;
  if let Some(e) = Event::from(event_type, event_num) {
    match e {
      Event::Interrupt(i) => {
        match INT_SEM.wait(t.clone(), i) {
          SemaphoreWaitResult::Acquired => {
            VOID
          },
          SemaphoreWaitResult::Enqueued => {
            thread_sleep(&t, crate::kernel::thread::Status::WaitForEvent);
            VOID_SCHEDULE
          },
        }
      }
      Event::ThreadExit(tid) => {
        let map = PARENT_WAIT_CHILD_MAP.lock();
        if let Some(vec) = map.get(&t.tid()) {
          if vec.contains(&tid) {
            VOID
          } else {
            Err(ERROR_HOLD_ON)
          }
        } else {
          Err(ERROR_HOLD_ON)
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
  ThreadExit(usize),
}

impl Event {
  pub fn from(event_type: usize, event_num: usize) -> Option<Self> {
    match event_type {
      EVENT_INTERRUPT => Some(Event::Interrupt(event_num)),
      EVENT_THREAD_EXIT => Some(Event::ThreadExit(event_num)),
      _ => None,
    }
  }
}

static PARENT_WAIT_CHILD_MAP: Mutex<BTreeMap<Tid, Vec<Tid>>> = Mutex::new(BTreeMap::new());

// called when a thread exits
pub fn thread_exit_signal(child_tid: Tid, parent_tid: Tid) {
  let mut map = PARENT_WAIT_CHILD_MAP.lock();
  if let Some(vec) = map.get_mut(&parent_tid) {
    vec.push(child_tid);
  } else {
    map.insert(parent_tid, vec![child_tid]);
  }
}

