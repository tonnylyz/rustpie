use alloc::collections::VecDeque;
use crate::lib::thread::Thread;

#[derive(Copy, Clone)]
pub enum Status {
  WAITING,
  ACTIVE,
  IDLE,
}

pub struct Notification {
  status: Status,
  value: usize,
  queue: VecDeque<Thread>,
}

impl Notification {
  pub fn signal(&mut self, value: usize) {
    match self.status {
      Status::WAITING => {
        if let Some(thread) = self.queue.pop_front() {
          thread.wake();
        } else {
          panic!("inconsistent status")
        }
        if self.queue.is_empty() {
          self.status = Status::IDLE;
        }
      }
      Status::ACTIVE => {
        self.value |= value;
      }
      Status::IDLE => {
        self.value = value;
        self.status = Status::ACTIVE;
      }
    }
  }

  pub fn wait(&mut self, thread: Thread) -> Option<usize> {
    match self.status {
      Status::WAITING => {
        self.queue.push_back(thread.clone());
        thread.sleep();
        None
      }
      Status::ACTIVE => {
        let value = self.value;
        self.status = Status::IDLE;
        self.value = 0;
        Some(value)
      }
      Status::IDLE => {
        self.queue.push_back(thread.clone());
        thread.sleep();
        self.status = Status::WAITING;
        None
      }
    }
  }

  pub fn poll(&mut self) -> Option<usize> {
    if self.value == 0 {
      return None;
    }
    match self.status {
      Status::WAITING => {
        let value = self.value;
        self.status = Status::IDLE;
        self.value = 0;
        Some(value)
      }
      Status::ACTIVE => {
        None
      }
      Status::IDLE => {
        None
      }
    }
  }
}