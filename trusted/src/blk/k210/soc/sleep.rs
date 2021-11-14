//! Utilities for sleeping short timespans
use super::sysctl;

pub fn cycle_sleep(n: usize) {
  let mut start = 0;
  loop {
    start += 1;
    if start > n {
      break;
    }
  }
}

pub fn usleep(n: usize) {
  let freq = sysctl::clock_get_freq(sysctl::clock::CPU) as usize;
  cycle_sleep(freq * n / 1000000);
}
