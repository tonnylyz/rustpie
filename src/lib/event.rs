use crate::driver::Interrupt;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Event {
  Interrupt(Interrupt),
  PageFault,
}

impl Into<usize> for Event {
  fn into(self) -> usize {
    match self {
      Event::Interrupt(u) => { u }
      Event::PageFault => { 0 }
    }
  }
}

impl From<usize> for Event {
  fn from(u: usize) -> Self {
    match u {
      0 => Event::PageFault,
      i => Event::Interrupt(i),
    }
  }
}