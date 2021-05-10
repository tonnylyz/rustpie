use spin::barrier::Barrier;

use crate::board::BOARD_CORE_NUMBER;

static CORES_BARRIER: spin::Barrier = Barrier::new(BOARD_CORE_NUMBER);

pub fn barrier() {
  CORES_BARRIER.wait();
}

#[inline(always)]
pub fn round_up(addr: usize, n: usize) -> usize {
  (addr + n - 1) & !(n - 1)
}

#[inline(always)]
pub fn round_down(addr: usize, n: usize) -> usize {
  addr & !(n - 1)
}

