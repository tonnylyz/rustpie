use alloc::collections::VecDeque;

use spin::{Mutex, Once};

use crate::kernel::thread::Thread;

use super::{traits::ArchTrait, interrupt::InterProcessorInterruptController};

pub struct RoundRobinScheduler {
  run_queue: Mutex<VecDeque<Thread>>,
  run_counts: Mutex<[usize; crate::board::BOARD_CORE_NUMBER]>,
}

impl RoundRobinScheduler {
  fn new() -> Self {
    RoundRobinScheduler {
      run_queue: Mutex::new(VecDeque::new()),
      run_counts: Mutex::new([0; crate::board::BOARD_CORE_NUMBER])
    }
  }

  pub fn add_front(&self, thread: Thread) {
    let mut inner = self.run_queue.lock();
    inner.push_front(thread);
  }

  fn least_busy_cpu(&self) -> usize {
    let mut min_count = usize::MAX;
    let mut min_cpu = 0;
    let counts = self.run_counts.lock();
    for i in 0..crate::board::BOARD_CORE_NUMBER {
      if counts[i] < min_count {
        min_count = counts[i];
        min_cpu = i;
      }
    }
    min_cpu
  }

  pub fn add(&self, thread: Thread) {
    let mut inner = self.run_queue.lock();
    inner.push_back(thread);
    drop(inner);
    // inform CPU to run
    let target = self.least_busy_cpu();
    if target != crate::arch::Arch::core_id() {
      crate::driver::INTERRUPT_CONTROLLER.send_to_one(super::interrupt::InterProcessInterrupt::IPI0, target);
    }
  }

  pub fn pop(&self) -> Option<Thread> {
    let mut counts = self.run_counts.lock();
    counts[crate::arch::Arch::core_id()] += 1;
    let mut inner = self.run_queue.lock();
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
