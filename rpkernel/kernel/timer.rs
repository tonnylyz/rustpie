#[allow(dead_code)]
const TIMER_SEC_TO_MS: usize = 1000;
#[allow(dead_code)]
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

#[allow(dead_code)]
pub fn udelay(us: usize) {
  let now = current_us();
  loop {
    if current_us() > (now + us) {
      break;
    }
  }
}

#[allow(dead_code)]
pub fn mdelay(ms: usize) {
  let now = current_ms();
  loop {
    if current_ms() > (now + ms) {
      break;
    }
  }
}

pub fn interrupt() {
  crate::kernel::cpu::cpu().tick(true);
}

#[cfg(target_arch = "aarch64")]
#[allow(dead_code)]
pub fn current_cycle() -> usize {
  let r;
  unsafe {
    core::arch::asm!("mrs {}, pmccntr_el0", out(reg) r);
  }
  r
}

#[cfg(target_arch = "riscv64")]
#[allow(dead_code)]
pub fn current_cycle() -> usize {
  let r;
  unsafe {
    core::arch::asm!("rdcycle {}", out(reg) r);
  }
  r
}