use cortex_a::regs::{CNTFRQ_EL0, RegisterReadOnly, CNTPCT_EL0};

const TIMER_TICK_MS: u64 = 100;

pub fn next() {
  use cortex_a::regs::*;
  let freq = CNTFRQ_EL0.get();
  let count = TIMER_TICK_MS * freq / 1000;
  CNTV_TVAL_EL0.set(count);
  CNTV_CTL_EL0.write(CNTV_CTL_EL0::ENABLE.val(1) + CNTV_CTL_EL0::IMASK.val(0));
}

pub fn frequency() -> usize {
  CNTFRQ_EL0.get() as usize
}

pub fn counter() -> usize {
  CNTPCT_EL0.get() as usize
}

pub fn init() {
  next();
}
