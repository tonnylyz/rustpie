const TIMER_SEC_TO_MS: usize = 1000;
const TIMER_SEC_TO_US: usize = 1000000;

#[allow(dead_code)]
pub fn current_us() -> usize {
  let count = crate::driver::timer::counter();
  let freq = crate::driver::timer::frequency();
  count * TIMER_SEC_TO_US / freq
}

#[allow(dead_code)]
pub fn current_ms() -> usize {
  let count = crate::driver::timer::counter();
  let freq = crate::driver::timer::frequency();
  count * TIMER_SEC_TO_MS / freq
}

#[allow(dead_code)]
pub fn current_sec() -> usize {
  let count = crate::driver::timer::counter();
  let freq = crate::driver::timer::frequency();
  count / freq
}

pub fn interrupt() {
  crate::driver::timer::next();
  crate::current_cpu().schedule();
}
