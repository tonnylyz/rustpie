use alloc::vec::Vec;
use core::ops::Range;

use crate::Address;
use crate::driver::gic::INT_TIMER;
use crate::kernel::device::Device;
use crate::kernel::interrupt::InterruptController;
use crate::kernel::traits::ArchTrait;

pub const BOARD_DEVICE_MEMORY_RANGE: Range<usize> = 0x0000_0000..0x4000_0000;
pub const BOARD_NORMAL_MEMORY_RANGE: Range<usize> = 0x4000_0000..0x1_0000_0000;

static CPU_NUMBER: spin::Once<usize> = spin::Once::new();
pub fn cpu_number() -> usize { *CPU_NUMBER.get().unwrap() }

pub fn init(fdt: usize) -> (Range<usize>, Range<usize>) {
  // println!("FDT phyaddr {:x}", fdt);
  let fdt = unsafe { core::slice::from_raw_parts(fdt.pa2kva() as *const u8, 1048576) };
  let fdt = fdt::Fdt::new(fdt).unwrap();
  // println!("FDT model {}", fdt.root().model());
  // println!("FDT compatible {}", fdt.root().compatible().first());

  CPU_NUMBER.call_once(|| { fdt.cpus().count() });
  for cpu in fdt.cpus() {
    let mpidr = cpu.ids().first();
    assert!(mpidr < 0xff);
    assert!(mpidr < crate::MAX_CPU_NUMBER);
    assert_eq!(cpu.property("enable-method").unwrap().as_str().unwrap(), "psci");
  }
  
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

  let memory_node = fdt.memory();
  let mut region_iter = memory_node.regions();
  let first_region = region_iter.next().expect("require at least one memory region");
  let memory_start = first_region.starting_address as usize;
  let memory_range = memory_start..(memory_start + first_region.size.unwrap());

  assert!(memory_range.contains(&kernel_end));
  assert!(memory_range.contains(&heap_start));
  assert!(memory_range.contains(&heap_end));
  assert!(memory_range.contains(&paged_start));
  let paged_end = memory_range.end;
  assert!(BOARD_NORMAL_MEMORY_RANGE.contains(&paged_end));

  let chosen = fdt.chosen();
  if let Some(stdout) = chosen.stdout() {
    println!("stdout {}", stdout.name);
  }

  crate::driver::uart::init();

  (heap_start..heap_end, paged_start..paged_end)
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
}

fn mpidr_to_linear(mpidr: u64) -> usize {
  assert_eq!(mpidr & 0x00ff00, 0);
  assert_eq!(mpidr & 0xff0000, 0);
  let id = (mpidr & 0xff) as usize;
  assert!(id < crate::MAX_CPU_NUMBER);
  id
}

fn linear_to_mpidr(id: usize) -> u64 {
  assert!(id < crate::MAX_CPU_NUMBER);
  id as u64
}

pub fn core_id() -> usize {
  mpidr_to_linear(crate::arch::Arch::raw_arch_id() as u64)
}

pub fn launch_other_cores() {
  extern "C" {
    fn KERNEL_ENTRY();
  }
  let core_id = mpidr_to_linear(crate::arch::Arch::raw_arch_id() as u64);
  for i in 0..crate::cpu_number() {
    if i != core_id {
      crate::driver::psci::cpu_on(linear_to_mpidr(i), (KERNEL_ENTRY as usize).kva2pa() as u64, 0);
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
