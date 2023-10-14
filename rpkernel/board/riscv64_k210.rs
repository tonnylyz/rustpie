use alloc::vec::Vec;
use core::ops::Range;
use tock_registers::interfaces::{Readable, Writeable};

use crate::kernel::traits::Address;
use crate::kernel::device::Device;
use crate::kernel::interrupt::InterruptController;

pub fn cpu_number() -> usize { 1 }

pub fn init(_fdt: usize) -> (Range<usize>, Range<usize>) {

  extern "C" {
    // Note: link-time label, see linker.ld
    fn KERNEL_END();
  }
  use crate::arch::PAGE_SIZE;
  let kernel_end = crate::util::round_up((KERNEL_END as usize).kva2pa(), PAGE_SIZE);
  let heap_start = kernel_end;
  let heap_end = heap_start + crate::mm::config::CONFIG_NON_PAGED_MEMORY_SIZE;
  let paged_start = heap_end;
  assert_eq!(kernel_end % PAGE_SIZE, 0);
  assert_eq!(heap_start % PAGE_SIZE, 0);
  assert_eq!(heap_end % PAGE_SIZE, 0);

  let memory_range = 0x8000_0000..0x8060_0000;
  
  assert!(memory_range.contains(&kernel_end));
  assert!(memory_range.contains(&heap_start));
  assert!(memory_range.contains(&heap_end));
  assert!(memory_range.contains(&paged_start));
  let paged_end = memory_range.end;
  
  crate::driver::uart::init();
  (heap_start..heap_end, paged_start..paged_end)
}

pub fn init_per_core() {
  crate::driver::timer::init();
  crate::driver::INTERRUPT_CONTROLLER.init();
  use riscv::regs::SIE;
  // enable
  //    * timer interrupt
  //    * external interrupt (from PLIC)
  //    * software interrupt (IPI)
  SIE.write(SIE::STIE::SET + SIE::SEIE::SET + SIE::SSIE::SET);
  info!("SIE val {:b}", SIE.get());
}

pub fn core_id() -> usize {
  0
}

pub fn launch_other_cores() {
  // TODO: enable hart 1 in K210
  // extern "C" {
  //   fn KERNEL_ENTRY();
  // }
  // let _ = crate::driver::hsm::hart_start(1, (KERNEL_ENTRY as usize).kva2pa(), 0);
}

#[no_mangle]
pub unsafe extern "C" fn hart_spin(core_id: usize) {
  // Note: K210 boot hart is always hart#0
  crate::main(core_id, 0);
}

pub fn devices() -> Vec<Device> {
  vec![
    Device::new("GPIOHS", vec![0x3800_1000..0x3800_2000], vec![]),
    Device::new("SPI0", vec![0x5200_0000..0x5200_1000], vec![]),
    Device::new("DMAC", vec![0x5000_0000..0x5000_1000], vec![]),
    Device::new("SYSCTL", vec![0x5044_0000..0x5044_1000], vec![]),
    Device::new("FPIOA", vec![0x502B_0000..0x502B_1000], vec![]),
  ]
}
