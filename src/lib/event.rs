use crate::driver::Interrupt;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Event {
  Interrupt(Interrupt),
  PageFault,
}
