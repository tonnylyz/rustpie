use alloc::collections::VecDeque;

use spin::Mutex;

use crate::kernel::thread::{Thread, thread_wake};

struct Inner {
  value: usize,
  queue: VecDeque<Thread>,
}

pub struct Semaphore {
  inner: Mutex<Inner>,
}

pub enum SemaphoreWaitResult {
  Acquired,
  Enqueued,
}

impl Semaphore {
  pub fn new() -> Self {
    Semaphore {
      inner: Mutex::new(Inner {
        value: 0,
        queue: VecDeque::new(),
      })
    }
  }

  pub fn wait(&self, t: Thread) -> SemaphoreWaitResult {
    let mut inner = self.inner.lock();
    if inner.value == 0 {
      inner.queue.push_back(t);
      SemaphoreWaitResult::Enqueued
    } else {
      inner.value -= 1;
      SemaphoreWaitResult::Acquired
    }
  }

  pub fn signal(&self) {
    let mut inner = self.inner.lock();
    if inner.value != 0 {
      inner.value += 1;
    } else {
      if let Some(t) = inner.queue.pop_front() {
        // wait for thread to be scheduled off any cpu
        loop {
          if t.running_cpu().is_none() {
            break;
          }
        }
        thread_wake(&t);
      } else {
        inner.value += 1;
      }
    }
  }
}

