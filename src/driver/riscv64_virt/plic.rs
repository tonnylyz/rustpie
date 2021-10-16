use crate::lib::interrupt::InterruptController;
use crate::lib::traits::ArchTrait;

// platform level interrupt controller
// https://github.com/riscv/riscv-plic-spec/blob/master/riscv-plic.adoc
const PLIC_BASE_ADDR: usize = 0xffff_ffff_0000_0000 + 0x0c00_0000;

const PLIC_SUPERVISOR_ENABLE_ADDR: usize = PLIC_BASE_ADDR + 0x2080;
// by 0x100
const PLIC_SUPERVISOR_PRIORITY_ADDR: usize = PLIC_BASE_ADDR + 0x201000;
// by 0x2000
const PLIC_SUPERVISOR_CLAIM_ADDR: usize = PLIC_BASE_ADDR + 0x201004;
// by 0x2000

pub struct Plic;

impl InterruptController for Plic {
  fn init(&self) {
    unsafe {
      let core_id = crate::arch::Arch::core_id();
      // set priority threshold
      ((PLIC_SUPERVISOR_PRIORITY_ADDR + core_id * 0x2000) as *mut u32).write(0);
    }
  }

  fn enable(&self, i: Interrupt) {
    let core_id = crate::arch::Arch::core_id();
    let reg = PLIC_SUPERVISOR_ENABLE_ADDR + core_id * 100 + (i / 32 * 4);
    unsafe {
      let val = (reg as *const u32).read();
      (reg as *mut u32).write(val | (1 << (i % 32)) as u32);
      // also set priority to 1
      ((PLIC_BASE_ADDR + i * 4) as *mut u32).write(1);
    }
  }

  fn disable(&self, i: Interrupt) {
    let core_id = crate::arch::Arch::core_id();
    let reg = PLIC_SUPERVISOR_ENABLE_ADDR + core_id * 100 + (i / 32 * 4);
    unsafe {
      let val = (reg as *const u32).read();
      (reg as *mut u32).write(val & !((1 << (i % 32)) as u32));
    }
  }

  fn fetch(&self) -> Option<Interrupt> {
    let core_id = crate::arch::Arch::core_id();
    let reg = PLIC_SUPERVISOR_CLAIM_ADDR + core_id * 0x2000;
    let int = unsafe {
      (reg as *mut u32).read() as usize
    };
    if int == 0 {
      None
    } else {
      Some(int)
    }
  }

  fn finish(&self, int: Interrupt) {
    let core_id = crate::arch::Arch::core_id();
    let reg = PLIC_SUPERVISOR_CLAIM_ADDR + core_id * 0x2000;
    unsafe { (reg as *mut u32).write(int as u32); }
  }
}

pub static INTERRUPT_CONTROLLER: Plic = Plic {};

pub type Interrupt = usize;
