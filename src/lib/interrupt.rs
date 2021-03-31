use crate::driver::Interrupt;

pub trait InterruptController {
  fn init(&self);

  fn enable(&self, int: Interrupt);
  fn disable(&self, int: Interrupt);

  fn fetch(&self) -> Option<Interrupt>;
  fn finish(&self, int: Interrupt);
}
