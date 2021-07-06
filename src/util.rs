use spin::barrier::Barrier;

use crate::arch::CoreId;
use crate::board::BOARD_CORE_NUMBER;
use crate::lib::cpu::Core;
use crate::lib::thread::Thread;
use crate::lib::traits::ArchTrait;

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

#[inline(always)]
pub fn core_id() -> CoreId {
  crate::arch::Arch::core_id()
}

#[inline(always)]
pub fn current_cpu() -> &'static mut Core {
  crate::lib::cpu::cpu()
}

#[inline(always)]
pub fn current_thread() -> Thread {
  crate::current_cpu().running_thread().unwrap()
}
