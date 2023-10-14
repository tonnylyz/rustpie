use alloc::vec::Vec;
use riscv::regs::SSCRATCH;
use core::ops::Range;
use core::sync::atomic::{AtomicBool, Ordering};
use tock_registers::interfaces::{Readable, Writeable};

use crate::MAX_CPU_NUMBER;
use crate::kernel::device::Device;
use crate::kernel::interrupt::InterruptController;
use crate::kernel::traits::*;

static CPU_NUMBER: spin::Once<usize> = spin::Once::new();
pub fn cpu_number() -> usize { *CPU_NUMBER.get().unwrap() }

pub fn init(fdt: usize) -> (Range<usize>, Range<usize>) {
  // println!("FDT phyaddr {:x}", fdt);
  let fdt = unsafe { core::slice::from_raw_parts(fdt.pa2kva() as *const u8, 8192) };
  let fdt = fdt::Fdt::new(fdt).unwrap();
  // println!("FDT model {}", fdt.root().model());
  // println!("FDT compatible {}", fdt.root().compatible().first());

  CPU_NUMBER.call_once(|| { fdt.cpus().count() });

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

  let chosen = fdt.chosen();
  if let Some(stdout) = chosen.stdout() {
    println!("It would write stdout to: {}", stdout.name);
  }

  let soc = fdt.find_node("/soc");
  if let Some(soc) = soc {
      for child in soc.children() {
        println!("soc/{}", child.name);
      }
  }
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
  SIE.write(
    SIE::STIE::SET
   + SIE::SEIE::SET
   + SIE::SSIE::SET
  );
  info!("SIE val {:b}", SIE.get());
}

pub fn core_id() -> usize {
  // Note: a pointer to hart_id is stored in sscratch
  unsafe {
    ((SSCRATCH.get() as usize) as *const usize).read()
  }
}

pub fn launch_other_cores() {
  HART_SPIN.store(true, Ordering::Relaxed);
}


static HART_SPIN: AtomicBool = AtomicBool::new(false);

#[no_mangle]
pub unsafe extern "C" fn hart_spin(core_id: usize, fdt: usize) {
  extern "C" {
    fn KERNEL_ENTRY();
  }
  static mut FDT: usize = 0;
  static mut HART_BOOT: usize = 0xffff_ffff;
  if HART_BOOT == 0xffff_ffff {
    HART_BOOT = core_id;
    FDT = fdt;
    for i in 0..MAX_CPU_NUMBER {
      if i != core_id {
        let _ = crate::driver::hsm::hart_start(i, (KERNEL_ENTRY as usize).kva2pa(), 0);
      }
    }
  }

  if core_id == 0 {
    crate::main(core_id, FDT);
  }
  while !HART_SPIN.load(Ordering::Acquire) {}
  crate::main(core_id, FDT);
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
