// use spin::barrier::Barrier;

use alloc::sync::Arc;
use core::sync::atomic;
use core::sync::atomic::{AtomicUsize, Ordering};
use spin::Mutex;
use crate::board::BOARD_CORE_NUMBER;

static COUNT: AtomicUsize = AtomicUsize::new(0);

pub fn barrier() {
  unsafe {
    let count = COUNT.fetch_add(1, Ordering::AcqRel);
    let next_count = round_up(count + 1, BOARD_CORE_NUMBER);
    loop {
      if COUNT.load(Ordering::Acquire) >= next_count {
        break;
      }
    }
  }
}

#[inline(always)]
pub fn round_up(addr: usize, n: usize) -> usize {
  (addr + n - 1) & !(n - 1)
}

#[inline(always)]
pub fn round_down(addr: usize, n: usize) -> usize {
  addr & !(n - 1)
}

