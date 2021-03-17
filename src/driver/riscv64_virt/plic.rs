use crate::arch::ArchTrait;
use crate::driver::mmio::*;
use crate::lib::interrupt::{InterruptController, InterruptNo};

// platform level interrupt controller
// https://github.com/riscv/riscv-plic-spec/blob/master/riscv-plic.adoc
const PLIC_BASE_ADDR: usize = 0xffff_ffff_0000_0000 + 0x0c00_0000;

const PLIC_SUPERVISOR_ENABLE_ADDR: usize = PLIC_BASE_ADDR + 0x2080;
// by 0x100
const PLIC_SUPERVISOR_PRIORITY_ADDR: usize = PLIC_BASE_ADDR + 0x201000;
// by 0x2000
const PLIC_SUPERVISOR_CLAIM_ADDR: usize = PLIC_BASE_ADDR + 0x201004;
// by 0x2000

const PLIC_IRQ_VIRTIO: InterruptNo = InterruptNo::Numbered(1);
const PLIC_IRQ_UART: InterruptNo = InterruptNo::Numbered(10);

pub struct Plic;

impl InterruptController for Plic {
  fn init(&self) {
    unsafe {
      let core_id = crate::arch::Arch::core_id();
      // set priority threshold
      write_word(PLIC_SUPERVISOR_PRIORITY_ADDR + core_id * 0x2000, 0);
    }
  }

  fn enable(&self, int: InterruptNo) {
    match int {
      InterruptNo::Timer => {panic!("PLIC does not manage timer irq")}
      InterruptNo::Numbered(i) => {
        let core_id = crate::arch::Arch::core_id();
        let reg = PLIC_SUPERVISOR_ENABLE_ADDR + core_id * 100 + (i / 32 * 4);
        unsafe {
          let val = read_word(reg);
          write_word(reg, val | (1 << (i % 32)) as u32);
          // also set priority to 1
          write_word(PLIC_BASE_ADDR + i * 4, 1);
        }
      }
    }
  }

  fn disable(&self, int: InterruptNo) {
    match int {
      InterruptNo::Timer => {panic!("PLIC does not manage timer irq")}
      InterruptNo::Numbered(i) => {
        let core_id = crate::arch::Arch::core_id();
        let reg = PLIC_SUPERVISOR_ENABLE_ADDR + core_id * 100 + (i / 32 * 4);
        unsafe {
          let val = read_word(reg);
          write_word(reg, val & !((1 << (i % 32)) as u32));
        }
      }
    }
  }

  fn fetch(&self) -> Option<InterruptNo> {
    let core_id = crate::arch::Arch::core_id();
    let reg = PLIC_SUPERVISOR_CLAIM_ADDR + core_id * 0x2000;
    let int = unsafe {
      read_word(reg) as usize
    };
    if int == 0 {
      None
    } else {
      Some(InterruptNo::Numbered(int))
    }
  }

  fn finish(&self, int: InterruptNo) {
    let core_id = crate::arch::Arch::core_id();
    let reg = PLIC_SUPERVISOR_CLAIM_ADDR + core_id * 0x2000;

    match int {
      InterruptNo::Timer => {panic!("PLIC does not manage timer irq")}
      InterruptNo::Numbered(i) => {
        unsafe { write_word(reg, i as u32); }
      }
    }
  }
}

pub const INT_TIMER: InterruptNo = InterruptNo::Timer;

pub static INTERRUPT_CONTROLLER: Plic = Plic{};
