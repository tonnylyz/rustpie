use alloc::vec::Vec;
use core::ops::Range;

use crate::Address;
use crate::driver::gic::INT_TIMER;
use crate::kernel::device::Device;
use crate::kernel::interrupt::InterruptController;
use crate::kernel::traits::ArchTrait;

pub const BOARD_CORE_NUMBER: usize = 4;
pub const BOARD_NORMAL_MEMORY_RANGE: Range<usize> = 0x8000_0000..0xf000_0000;
pub const BOARD_DEVICE_MEMORY_RANGE: Range<usize> = 0x0000_0000..0x8000_0000;

pub fn init() {
  crate::driver::uart::init();
}

pub fn init_per_core() {
  use aarch64_cpu::registers::*;
  use tock_registers::interfaces::Writeable;
  DAIF.write(DAIF::I::Masked);
  crate::driver::INTERRUPT_CONTROLLER.init();
  crate::driver::INTERRUPT_CONTROLLER.enable(INT_TIMER);
  crate::driver::timer::init();
  let mut pmcr: u32;
  let pmcntenclr = u32::MAX as u64;
  let pmcntenset = 1u64 << 31;
  let pmuserenr = 1u64 << 2 | 1u64;
  unsafe {
    core::arch::asm!("msr pmcntenclr_el0, {}", in(reg) pmcntenclr);

    core::arch::asm!("mrs {:x}, pmcr_el0", out(reg) pmcr);
    pmcr &= !(1u32 << 3);
    core::arch::asm!("msr pmcr_el0, {:x}", in(reg) pmcr);

    core::arch::asm!("mrs {:x}, pmcr_el0", out(reg) pmcr);
    pmcr |= (1u32 << 1) | (1u32 << 2);
    core::arch::asm!("msr pmcr_el0, {:x}", in(reg) pmcr);

    core::arch::asm!("mrs {:x}, pmcr_el0", out(reg) pmcr);
    pmcr |= 1;
    core::arch::asm!("msr pmcr_el0, {:x}", in(reg) pmcr);

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
      crate::driver::psci::cpu_on((i as u64) | 0x80000100, (KERNEL_ENTRY as usize).kva2pa() as u64, 0);
    }
  }
}

pub fn devices() -> Vec<Device> {
  vec![]
}
