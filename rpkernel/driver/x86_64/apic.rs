pub struct Apic;

pub static INTERRUPT_CONTROLLER: Once<Apic> = Once::new();

pub type Interrupt = usize;

use spin::Once;

use crate::kernel::interrupt::{
  InterProcessInterrupt as IPI, InterProcessorInterruptController, InterruptController,
};

impl InterruptController for Once<Apic> {
  fn init(&self) {
    todo!()
  }

  fn enable(&self, int: Interrupt) {
    todo!()
  }

  fn disable(&self, int: Interrupt) {
    todo!()
  }

  fn fetch(&self) -> Option<(Interrupt, usize)> {
    todo!()
  }

  fn finish(&self, int: Interrupt) {
    todo!()
  }
}

impl InterProcessorInterruptController for Once<Apic> {
  fn send_to_one(&self, irq: IPI, target: usize) {
    todo!()
  }

  fn send_to_multiple(&self, irq: IPI, target_mask: usize) {
    todo!()
  }
}
