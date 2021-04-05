use crate::driver::Interrupt;
use alloc::collections::BTreeMap;
use crate::lib::thread::Thread;
use spin::Mutex;

pub trait InterruptController {
  fn init(&self);

  fn enable(&self, int: Interrupt);
  fn disable(&self, int: Interrupt);

  fn fetch(&self) -> Option<Interrupt>;
  fn finish(&self, int: Interrupt);
}

pub struct InterruptWait(Mutex<BTreeMap<Interrupt, Thread>>);

pub static INTERRUPT_WAIT: InterruptWait = InterruptWait(Mutex::new(BTreeMap::new()));

pub enum Error {
  AlreadyWaiting,
}

impl InterruptWait {
  pub fn add(&self, t: Thread, i: Interrupt) -> Result<(), Error> {
    let mut map = self.0.lock();
    if let Some(_) = map.get(&i) {
      Err(Error::AlreadyWaiting)
    } else {
      map.insert(i, t.clone());
      Ok(())
    }
  }

  pub fn get(&self, i: Interrupt) -> Option<Thread> {
    let map = self.0.lock();
    map.get(&i).cloned()
  }
}

