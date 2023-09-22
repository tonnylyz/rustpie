use alloc::vec::Vec;
use core::ops::Range;

use crate::kernel::device::Device;
use crate::kernel::interrupt::InterruptController;
use crate::kernel::traits::ArchTrait;

pub const BOARD_CORE_NUMBER: usize = 1;

pub const BOARD_NORMAL_MEMORY_RANGE: Range<usize> = 0x8000_0000..0x8060_0000;
#[allow(dead_code)]
pub const BOARD_DEVICE_MEMORY_RANGE: Range<usize> = 0x0000_0000..0x8000_0000;

pub fn init() {
  crate::driver::uart::init();
}

pub fn init_per_core() {
  crate::driver::timer::init();
  crate::arch::Arch::exception_init();
  crate::driver::INTERRUPT_CONTROLLER.init();
}


pub fn launch_other_cores() {
  // extern "C" {
  //   fn KERNEL_ENTRY();
  // }
  // let _ = crate::driver::hsm::hart_start(1, (KERNEL_ENTRY as usize).kva2pa(), 0);
}

#[no_mangle]
pub unsafe extern "C" fn hart_spin(core_id: usize) {
  crate::main(core_id);
}

pub fn devices() -> Vec<Device> {
  vec![
    Device::new(
      "GPIOHS",
      vec![
        0x3800_1000..0x3800_2000
      ],
      vec![]),
    Device::new(
      "SPI0",
      vec![
        0x5200_0000..0x5200_1000
      ],
      vec![]),
    Device::new(
      "DMAC",
      vec![
        0x5000_0000..0x5000_1000
      ],
      vec![]),
    Device::new(
      "SYSCTL",
      vec![
        0x5044_0000..0x5044_1000
      ],
      vec![]),
    Device::new(
      "FPIOA",
      vec![
        0x502B_0000..0x502B_1000
      ],
      vec![]),
  ]
}
