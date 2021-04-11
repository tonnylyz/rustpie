use crate::board::BOARD_CORE_NUMBER;
use spin::barrier::Barrier;

static CORES_BARRIER: spin::Barrier = Barrier::new(BOARD_CORE_NUMBER);

pub fn barrier() {
  CORES_BARRIER.wait();
}
