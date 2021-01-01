use crate::arch::ArchTrait;
use crate::driver::mmio::write_byte;

const TIMER_DEFAULT_COUNT: u64 = 10000000;

pub fn next() {
  use cortex_a::regs::*;
  CNTP_TVAL_EL0.set(TIMER_DEFAULT_COUNT);
  CNTP_CTL_EL0.write(CNTP_CTL_EL0::ENABLE.val(1) + CNTP_CTL_EL0::IMASK.val(0));
}

pub fn init() {
  next();
}
