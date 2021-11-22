use alloc::collections::VecDeque;

use spin::{Mutex, Once};

use crate::lib::thread::Thread;

pub struct RoundRobinScheduler {
  inner: Mutex<VecDeque<Thread>>,
}

impl RoundRobinScheduler {
  fn new() -> Self {
    RoundRobinScheduler {
      inner: Mutex::new(VecDeque::new()),
    }
  }

  pub fn add_front(&self, thread: Thread) {
    let mut inner = self.inner.lock();
    inner.push_front(thread);
  }

  pub fn add(&self, thread: Thread) {
    let mut inner = self.inner.lock();
    inner.push_back(thread);
  }

  pub fn pop(&self) -> Option<Thread> {
    let mut inner = self.inner.lock();
    inner.pop_front()
  }
}

static SCHEDULER: Once<RoundRobinScheduler> = Once::new();

pub fn scheduler() -> &'static RoundRobinScheduler {
  if let Some(s) = SCHEDULER.get() {
    s
  } else {
    SCHEDULER.call_once(|| RoundRobinScheduler::new())
  }
}
