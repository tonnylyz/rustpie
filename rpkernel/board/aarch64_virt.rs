use alloc::vec::Vec;
use core::ops::Range;

use crate::Address;
use crate::driver::gic::INT_TIMER;
use crate::kernel::device::Device;
use crate::kernel::interrupt::InterruptController;
use crate::kernel::traits::ArchTrait;

pub const BOARD_CORE_NUMBER: usize = 4;
pub const BOARD_NORMAL_MEMORY_RANGE: Range<usize> = 0x4000_0000..0x8000_0000;
pub const BOARD_DEVICE_MEMORY_RANGE: Range<usize> = 0x0000_0000..0x4000_0000;

pub fn init() {
  crate::driver::uart::init();
}

pub fn init_per_core() {
  use aarch64_cpu::registers::*;
  use tock_registers::interfaces::Writeable;
  DAIF.write(DAIF::I::Masked);
  crate::driver::INTERRUPT_CONTROLLER.init();
  crate::driver::INTERRUPT_CONTROLLER.enable(INT_TIMER);
  for i in 0..16 {
    crate::driver::INTERRUPT_CONTROLLER.enable(i);
  }
  crate::driver::timer::init();
  let pmcr = 1u64;
  let pmcntenset = 1u64 << 32;
  let pmuserenr = 1u64 << 2 | 1u64;
  unsafe {
    core::arch::asm!("msr pmcr_el0, {}", in(reg) pmcr);
    core::arch::asm!("msr pmcntenset_el0, {}", in(reg) pmcntenset);
    core::arch::asm!("msr pmuserenr_el0, {}", in(reg) pmuserenr);
  }
}

pub fn launch_other_cores() {
  extern "C" {
    fn KERNEL_ENTRY();
  }
  let core_id = crate::arch::Arch::core_id();
  for i in 0..BOARD_CORE_NUMBER {
    if i != core_id {
      crate::driver::psci::cpu_on(i as u64, (KERNEL_ENTRY as usize).kva2pa() as u64, 0);
    }
  }
}

// virtio_mmio@a000000 {
//   dma-coherent;
//   interrupts = <0x00 0x10 0x01>;
//   reg = <0x00 0xa000000 0x00 0x200>;
//   compatible = "virtio,mmio";
// };

// pl031@9010000 {
//   clock-names = "apb_pclk";
//   clocks = <0x8000>;
//   interrupts = <0x00 0x02 0x04>;
//   reg = <0x00 0x9010000 0x00 0x1000>;
//   compatible = "arm,pl031\0arm,primecell";
// };

// pl011@9000000 {
//   clock-names = "uartclk\0apb_pclk";
//   clocks = <0x8000 0x8000>;
//   interrupts = <0x00 0x01 0x04>;
//   reg = <0x00 0x9000000 0x00 0x1000>;
//   compatible = "arm,pl011\0arm,primecell";
// };

pub fn devices() -> Vec<Device> {
  vec![
    Device::new("virtio_blk", vec![0xa000000..0xa000200], vec![0x10 + 32]),
    Device::new("pl031", vec![0x9010000..0x9011000], vec![]),
    Device::new("pl011", vec![0x9000000..0x9001000], vec![0x1 + 32]),
  ]
}
