use alloc::collections::BTreeMap;

use spin::Mutex;

use crate::driver::Interrupt;
use crate::kernel::semaphore::{Semaphore, SemaphoreWaitResult};
use crate::kernel::thread::Thread;

pub trait InterruptController {
  fn init(&self);

  fn enable(&self, int: Interrupt);
  fn disable(&self, int: Interrupt);

  fn fetch(&self) -> Option<(Interrupt, usize)>;
  fn finish(&self, int: Interrupt);
}

#[derive(Debug, Copy, Clone)]
pub enum InterProcessInterrupt {
  IPI0,
}

pub trait InterProcessorInterruptController {
  fn send_to_one(&self, irq: InterProcessInterrupt, target: usize);
  fn send_to_multiple(&self, irq: InterProcessInterrupt, target_mask: usize);
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

// Routine that exception handler calls to handle external interrupt (SPI interrupts in AArch64 or PLIC interrupts in Riscv64)
pub fn interrupt(int: Interrupt) {
  trace!("external {}", int);
  INT_SEM.signal(int);
}

// Routine that exception handler calls to handle inter process interrupts
pub fn ipi_interrupt(ipi: InterProcessInterrupt, _src_cpu: usize) {
  match ipi {
    InterProcessInterrupt::IPI0 => {
      let this = crate::kernel::cpu::cpu();
      if this.running_idle() {
        this.tick(false);
      }
    }
  }
}
