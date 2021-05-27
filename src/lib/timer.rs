use crate::lib::cpu::CoreTrait;

const TIMER_SEC_TO_MS: usize = 1000;
const TIMER_SEC_TO_US: usize = 1000000;

pub fn current_us() -> usize {
  let count = crate::driver::timer::counter();
  let freq = crate::driver::timer::frequency();
  count * TIMER_SEC_TO_US / freq
}

pub fn current_ms() -> usize {
  let count = crate::driver::timer::counter();
  let freq = crate::driver::timer::frequency();
  count * TIMER_SEC_TO_MS / freq
}

pub fn current_sec() -> usize {
  let count = crate::driver::timer::counter();
  let freq = crate::driver::timer::frequency();
  count / freq
}

pub fn interrupt() {
  crate::driver::timer::next();
  crate::lib::cpu::current().schedule();
}
