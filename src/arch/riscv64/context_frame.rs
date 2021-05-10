use core::fmt::Formatter;

use riscv::regs::*;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Riscv64ContextFrame {
  gpr: [u64; 32],
  sstatus: u64,
  sepc: u64,
}

static REG_ABI_NAMES: [&str; 32] = [
  "ZERO",
  "RA",
  "SP",
  "GP",
  "TP",
  "T0",
  "T1",
  "T2",
  "S0/FP",
  "S1",
  "A0",
  "A1",
  "A2",
  "A3",
  "A4",
  "A5",
  "A6",
  "A7",
  "S2",
  "S3",
  "S4",
  "S5",
  "S6",
  "S7",
  "S8",
  "S9",
  "S10",
  "S11",
  "T3",
  "T4",
  "T5",
  "T6",
];

impl core::fmt::Display for Riscv64ContextFrame {
  fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), core::fmt::Error> {
    for i in 0..32 {
      write!(f, "{:5}: {:016x}   ", REG_ABI_NAMES[i], self.gpr[i])?;
      if (i + 1) % 2 == 0 {
        write!(f, "\n")?;
      }
    }
    write!(f, "{:5}: {:016x}   ", "SSTAT", self.sstatus)?;
    writeln!(f, "{:5}: {:016x}   ", "EPC", self.sepc)?;
    Ok(())
  }
}

impl crate::lib::traits::ContextFrameTrait for Riscv64ContextFrame {
  fn new(pc: usize, sp: usize, arg: usize, privileged: bool) -> Self {
    let mut r = Riscv64ContextFrame {
      gpr: [0xdeadbeef_deadbeef; 32],
      sstatus: (SSTATUS::SD::SET + SSTATUS::FS.val(0b11) + if privileged { SSTATUS::SPP::Supervisor } else { SSTATUS::SPP::User } + SSTATUS::SPIE.val(1) + SSTATUS::SIE.val(0)).value,
      sepc: 0xdeadbeef_deadbeef,
    };
    r.set_exception_pc(pc);
    r.set_stack_pointer(sp);
    r.set_argument(arg);
    r
  }

  fn syscall_argument(&self, i: usize) -> usize {
    assert!(i <= 5);
    // a0 ~ a5 -> x10 ~ x15
    self.gpr[i + 10] as usize
  }

  fn syscall_number(&self) -> usize {
    // a7 -> x17
    self.gpr[17] as usize
  }

  fn set_syscall_return_value(&mut self, v: usize) {
    // a0 -> x10
    self.gpr[10] = v as u64;
  }

  fn exception_pc(&self) -> usize {
    self.sepc as usize
  }

  fn set_exception_pc(&mut self, pc: usize) {
    self.sepc = pc as u64;
  }

  fn stack_pointer(&self) -> usize {
    // sp -> x2
    self.gpr[2] as usize
  }

  fn set_stack_pointer(&mut self, sp: usize) {
    self.gpr[2] = sp as u64;
  }

  fn set_argument(&mut self, arg: usize) {
    self.gpr[10] = arg as u64;
  }

  fn gpr(&self, index: usize) -> usize {
    self.gpr[index] as usize
  }
}

