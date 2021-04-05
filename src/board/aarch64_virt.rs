use core::ops::Range;

use crate::driver::gic::INT_TIMER;
use crate::lib::interrupt::InterruptController;
use alloc::vec::Vec;
use crate::driver::Interrupt;
use crate::arch::Address;

pub const BOARD_CORE_NUMBER: usize = 1;
pub const BOARD_NORMAL_MEMORY_RANGE: Range<usize> = 0x4000_0000..0x8000_0000;
pub const BOARD_DEVICE_MEMORY_RANGE: Range<usize> = 0x0000_0000..0x4000_0000;

pub fn init() {
  crate::driver::uart::init();
  // crate::driver::common::virtio_blk::init();
}

pub fn init_per_core() {
  use cortex_a::regs::*;
  DAIF.write(DAIF::I::Masked);
  crate::driver::INTERRUPT_CONTROLLER.init();
  crate::driver::INTERRUPT_CONTROLLER.enable(INT_TIMER);
  crate::driver::timer::init();
}

pub fn launch_other_cores() {
  extern "C" {
    fn KERNEL_ENTRY();
  }
  let core_id = crate::core_id();
  for i in 0..BOARD_CORE_NUMBER {
    if i != core_id {
      crate::driver::psci::cpu_on(i as u64, (KERNEL_ENTRY as usize).kva2pa() as u64, 0);
    }
  }
}
