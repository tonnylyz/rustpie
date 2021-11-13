use core::ops::Range;
use core::sync::atomic::{AtomicBool, Ordering};

use spin::Mutex;

use crate::lib::traits::*;
use crate::lib::interrupt::InterruptController;
use alloc::vec::Vec;
use crate::lib::device::Device;

pub const BOARD_CORE_NUMBER: usize = 4;

pub const BOARD_NORMAL_MEMORY_RANGE: Range<usize> = 0x8000_0000..0xc000_0000;
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
  HART_SPIN.store(true, Ordering::Relaxed);
}


static HART_SPIN: AtomicBool = AtomicBool::new(false);
static HART_BOOT: Mutex<Option<usize>> = Mutex::new(None);

#[no_mangle]
pub unsafe extern "C" fn hart_spin(core_id: usize) {
  extern "C" {
    fn KERNEL_ENTRY();
  }
  let mut hart_boot = HART_BOOT.lock();
  if hart_boot.is_none() {
    *hart_boot = Some(core_id);
    drop(hart_boot);
    for i in 0..BOARD_CORE_NUMBER {
      if i != core_id {
        let _ = crate::driver::hsm::hart_start(i, (KERNEL_ENTRY as usize).kva2pa(), 0);
      }
    }
  } else {
    drop(hart_boot);
  }

  if core_id == 0 {
    crate::main(core_id);
  }
  while !HART_SPIN.load(Ordering::Relaxed) {}
  crate::main(core_id);
}

pub fn devices() -> Vec<Device> {
  vec![
    Device::new(
      "virtio_blk",
      vec![
        0x10001000..0x10002000
      ],
      vec![
        1
      ]),
    Device::new(
      "rtc",
      vec![
        0x101000..0x102000
      ],
      vec![]),
  ]
}
