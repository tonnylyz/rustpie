use spin::{Mutex, Once};

use crate::kernel::thread::Thread;

use super::{traits::ArchTrait, interrupt::InterProcessorInterruptController, cpu::{cpu_nth, cpu}};

pub struct SmpScheduler {
  run_counts: Mutex<[usize; crate::board::BOARD_CORE_NUMBER]>,
}

impl SmpScheduler {
  fn new() -> Self {
    SmpScheduler {
      run_counts: Mutex::new([0; crate::board::BOARD_CORE_NUMBER])
    }
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
    // inform CPU to run
    let target = self.least_busy_cpu();
    let mut counts = self.run_counts.lock();
    counts[target] += 1;
    if target != crate::arch::Arch::core_id() {
      let cpu = cpu_nth(target);
      cpu.enqueue_task(thread, false);
      crate::driver::INTERRUPT_CONTROLLER.send_to_one(super::interrupt::InterProcessInterrupt::IPI0, target);
    } else {
      let cpu = cpu();
      cpu.enqueue_task(thread, false);
    }
  }
}

static SCHEDULER: Once<SmpScheduler> = Once::new();

pub fn scheduler() -> &'static SmpScheduler {
  if let Some(s) = SCHEDULER.get() {
    s
  } else {
    SCHEDULER.call_once(|| SmpScheduler::new())
  }
}
