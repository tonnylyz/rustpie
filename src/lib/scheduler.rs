use crate::lib::thread::Thread;

#[derive(Copy, Clone)]
pub struct RoundRobinScheduler {
  counter: usize,
}

pub trait SchedulerTrait {
  fn schedule(&mut self) -> Option<Thread>;
}

impl SchedulerTrait for RoundRobinScheduler {
  fn schedule(&mut self) -> Option<Thread> {
    self.counter += 1;
    let list = crate::lib::thread::list();
    // println!("list {:#x?}", list);
    for i in (self.counter % list.len())..list.len() {
      let t = list[i].clone();
      if t.runnable() {
        if t.assign_to_current_core() {
          return Some(t.clone());
        }
      }
    }
    for i in 0..list.len() {
      let t = list[i].clone();
      if t.runnable() {
        if t.assign_to_current_core() {
          return Some(t.clone());
        }
      }
    }
    None
  }
}

impl RoundRobinScheduler {
  pub const fn new() -> Self {
    RoundRobinScheduler {
      counter: 0
    }
  }
}
