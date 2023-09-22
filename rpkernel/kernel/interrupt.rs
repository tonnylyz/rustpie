use alloc::collections::BTreeMap;

use spin::Mutex;

use crate::driver::Interrupt;
use crate::kernel::semaphore::{Semaphore, SemaphoreWaitResult};
use crate::kernel::thread::Thread;

pub trait InterruptController {
  fn init(&self);

  fn enable(&self, int: Interrupt);
  fn disable(&self, int: Interrupt);

  fn fetch(&self) -> Option<Interrupt>;
  fn finish(&self, int: Interrupt);
}

pub struct InterruptSemaphore(Mutex<BTreeMap<Interrupt, Semaphore>>);

pub static INT_SEM: InterruptSemaphore = InterruptSemaphore(Mutex::new(BTreeMap::new()));

impl InterruptSemaphore {
  pub fn wait(&self, t: Thread, i: Interrupt) -> SemaphoreWaitResult {
    let mut map = self.0.lock();
    if let Some(sem) = map.get(&i) {
      sem.wait(t)
    } else {
      let sem = Semaphore::new();
      sem.wait(t);
      map.insert(i, sem);
      SemaphoreWaitResult::Enqueued
    }
  }

  pub fn signal(&self, i: Interrupt) {
    let mut map = self.0.lock();
    if let Some(sem) = map.get(&i) {
      sem.signal();
    } else {
      let sem = Semaphore::new();
      sem.signal();
      map.insert(i, sem);
    }
  }
}

pub fn interrupt(int: Interrupt) {
  // info!("external {}", int);
  INT_SEM.signal(int);
}

