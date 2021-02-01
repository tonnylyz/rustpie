use riscv::regs::*;

use super::sbi::{SBI_EID_TIMER, SBI_FID_SET_TIMER, sbi_call};

const TIMER_DEFAULT_COUNT: usize = 250000;

pub fn next() {
  let time = TIME.get() as usize;
  sbi_call(SBI_EID_TIMER, SBI_FID_SET_TIMER, time + TIMER_DEFAULT_COUNT, 0, 0);
}

pub fn init() {
  next();
  SIE.write(SIE::STIE.val(1));
}
