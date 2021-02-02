use core::ops::Range;
use core::sync::atomic::{AtomicBool, Ordering};

use crate::arch::{ArchTrait, CoreTrait};
use crate::lib::current_core;
use spin::Mutex;

#[allow(dead_code)]
pub const BOARD_CORE_NUMBER: usize = 4;
#[allow(dead_code)]
pub const BOARD_PHYSICAL_ADDRESS_LIMIT: usize = 0xc000_0000;
#[allow(dead_code)]
pub const BOARD_NORMAL_MEMORY_RANGE: Range<usize> = 0x8000_0000..0xc000_0000;
#[allow(dead_code)]
pub const BOARD_DEVICE_MEMORY_RANGE: Range<usize> = 0x0000_0000..0x8000_0000;

pub fn init() {
  crate::driver::uart::init();
  crate::driver::plic::init();
}

pub fn init_per_core() {
  crate::driver::timer::init();
  crate::arch::Arch::exception_init();
  current_core().create_idle_thread();
}

pub fn launch_other_cores() {
  HART_SPIN.store(true, Ordering::Relaxed);
}

static HART_SPIN: AtomicBool = AtomicBool::new(false);
static HART_BOOT: Mutex<Option<usize>> = Mutex::new(None);
#[no_mangle]
pub unsafe extern "C" fn hart_spin(core_id: usize) {
  let mut hart_boot = HART_BOOT.lock();
  if hart_boot.is_none() {
    *hart_boot = Some(core_id);
    for i in 0..BOARD_CORE_NUMBER {
      if i != core_id {
        let _ = crate::driver::hsm::hart_start(i, 0x80200000, 0);
      }
    }
  }
  drop(hart_boot);

  if core_id == 0 {
    crate::main(core_id);
  }
  while !HART_SPIN.load(Ordering::Relaxed) {}
  crate::main(core_id);
}
