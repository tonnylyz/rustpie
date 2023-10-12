use alloc::vec::Vec;
use core::ops::Range;
use core::sync::atomic::{AtomicBool, Ordering};
use tock_registers::interfaces::{Readable, Writeable};

use crate::kernel::device::Device;
use crate::kernel::interrupt::InterruptController;
use crate::kernel::traits::*;

pub const BOARD_CORE_NUMBER: usize = 4;

pub const BOARD_NORMAL_MEMORY_RANGE: Range<usize> = 0x8000_0000..0xc000_0000;
#[allow(dead_code)]
pub const BOARD_DEVICE_MEMORY_RANGE: Range<usize> = 0x0000_0000..0x8000_0000;

pub fn init() {
  crate::driver::uart::init();
}

pub fn init_per_core() {
  crate::driver::timer::init();
  crate::driver::INTERRUPT_CONTROLLER.init();
  use riscv::regs::SIE;
  // enable
  //    * timer interrupt
  //    * external interrupt (from PLIC)
  //    * software interrupt (IPI)
  SIE.write(
    SIE::STIE::SET
   + SIE::SEIE::SET
   + SIE::SSIE::SET
  );
  info!("SIE val {:b}", SIE.get());
}

pub fn launch_other_cores() {
  unsafe {
    info!("boot hard is {}", HART_BOOT);
  }
  HART_SPIN.store(true, Ordering::Relaxed);
}


static HART_SPIN: AtomicBool = AtomicBool::new(false);
static mut HART_BOOT: usize = 0xffff_ffff;

#[no_mangle]
pub unsafe extern "C" fn hart_spin(core_id: usize) {
  extern "C" {
    fn KERNEL_ENTRY();
  }
  if HART_BOOT == 0xffff_ffff {
    HART_BOOT = core_id;
    for i in 0..BOARD_CORE_NUMBER {
      if i != core_id {
        let _ = crate::driver::hsm::hart_start(i, (KERNEL_ENTRY as usize).kva2pa(), 0);
      }
    }
  }

  if core_id == 0 {
    crate::main(core_id);
  }
  while !HART_SPIN.load(Ordering::Acquire) {}
  crate::main(core_id);
}

// virtio_mmio@10001000 {
//   interrupts = <0x01>;
//   interrupt-parent = <0x09>;
//   reg = <0x00 0x10001000 0x00 0x1000>;
//   compatible = "virtio,mmio";
// };

// rtc@101000 {
//   interrupts = <0x0b>;
//   interrupt-parent = <0x09>;
//   reg = <0x00 0x101000 0x00 0x1000>;
//   compatible = "google,goldfish-rtc";
// };

// serial@10000000 {
//   interrupts = <0x0a>;
//   interrupt-parent = <0x09>;
//   clock-frequency = "\08@";
//   reg = <0x00 0x10000000 0x00 0x100>;
//   compatible = "ns16550a";
// };

pub fn devices() -> Vec<Device> {
  vec![
    Device::new("virtio_blk", vec![0x10001000..0x10002000], vec![0x1]),
    Device::new("rtc", vec![0x101000..0x102000], vec![]),
    Device::new("serial", vec![0x10000000..0x10001000], vec![0xa]),
  ]
}
