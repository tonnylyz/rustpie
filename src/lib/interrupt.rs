use alloc::collections::BTreeMap;

use spin::Mutex;

use crate::driver::Interrupt;
use crate::lib::cpu::CoreTrait;
use crate::lib::event::Event;
use crate::lib::thread::Thread;
use crate::lib::thread::Status::{TsRunnable, TsWaitForInterrupt};

pub trait InterruptController {
  fn init(&self);

  fn enable(&self, int: Interrupt);
  fn disable(&self, int: Interrupt);

  fn fetch(&self) -> Option<Interrupt>;
  fn finish(&self, int: Interrupt);
}

pub struct InterruptWait(Mutex<BTreeMap<Interrupt, Option<Thread>>>);

pub static INTERRUPT_WAIT: InterruptWait = InterruptWait(Mutex::new(BTreeMap::new()));

pub enum Error {
  AlreadyWaiting,
  ThreadNotWaiting,
}

impl InterruptWait {
  pub fn add_yield(&self, t: Thread, i: Interrupt) -> Result<(), Error> {
    let mut map = self.0.lock();
    if let Some(_) = map.get(&i) {
      Err(Error::AlreadyWaiting)
    } else {
      map.insert(i, Some(t.clone()));
      Ok(())
    }
  }

  pub fn add_happened(&self, i: Interrupt) -> Result<(), Error> {
    let mut map = self.0.lock();
    if let Some(_) = map.get(&i) {
      Err(Error::AlreadyWaiting)
    } else {
      map.insert(i, None);
      Ok(())
    }
  }

  pub fn get(&self, i: Interrupt) -> Option<Thread> {
    let map = self.0.lock();
    match map.get(&i) {
      None => { None }
      Some(w) => { w.clone() }
    }
  }

  pub fn remove(&self, i: Interrupt) -> Result<(), Error> {
    let mut map = self.0.lock();
    match map.remove(&i) {
      None => Err(Error::ThreadNotWaiting),
      Some(_) => Ok(()),
    }
  }
}

pub fn interrupt(int: Interrupt) {
  info!("[IRQ] external {}", int);
  if let Some(t) = INTERRUPT_WAIT.get(int) {
    INTERRUPT_WAIT.remove(int);
    assert_eq!(t.status(), TsWaitForInterrupt);
    t.set_status(TsRunnable);
    crate::driver::timer::next();
    crate::lib::cpu::current().schedule();
  } else {
    INTERRUPT_WAIT.add_happened(int);
  }
}

