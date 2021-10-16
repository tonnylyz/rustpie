use core::fmt::Formatter;

use riscv::regs::*;
use unwind::registers::{Registers, Riscv64};
use crate::syscall::{Result as SyscallResult, SyscallOutRegisters};

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

impl Into<Registers> for Riscv64ContextFrame {
  fn into(self) -> Registers {
    let mut reg = Registers::default();
    reg[Riscv64::X0] = Some(self.gpr[0]);
    reg[Riscv64::X1] = Some(self.gpr[1]);
    reg[Riscv64::X2] = Some(self.gpr[2]);
    reg[Riscv64::X3] = Some(self.gpr[3]);
    reg[Riscv64::X4] = Some(self.gpr[4]);
    reg[Riscv64::X5] = Some(self.gpr[5]);
    reg[Riscv64::X6] = Some(self.gpr[6]);
    reg[Riscv64::X7] = Some(self.gpr[7]);
    reg[Riscv64::X8] = Some(self.gpr[8]);
    reg[Riscv64::X9] = Some(self.gpr[9]);
    reg[Riscv64::X10] = Some(self.gpr[10]);
    reg[Riscv64::X11] = Some(self.gpr[11]);
    reg[Riscv64::X12] = Some(self.gpr[12]);
    reg[Riscv64::X13] = Some(self.gpr[13]);
    reg[Riscv64::X14] = Some(self.gpr[14]);
    reg[Riscv64::X15] = Some(self.gpr[15]);
    reg[Riscv64::X16] = Some(self.gpr[16]);
    reg[Riscv64::X17] = Some(self.gpr[17]);
    reg[Riscv64::X18] = Some(self.gpr[18]);
    reg[Riscv64::X19] = Some(self.gpr[19]);
    reg[Riscv64::X20] = Some(self.gpr[20]);
    reg[Riscv64::X21] = Some(self.gpr[21]);
    reg[Riscv64::X22] = Some(self.gpr[22]);
    reg[Riscv64::X23] = Some(self.gpr[23]);
    reg[Riscv64::X24] = Some(self.gpr[24]);
    reg[Riscv64::X25] = Some(self.gpr[25]);
    reg[Riscv64::X26] = Some(self.gpr[26]);
    reg[Riscv64::X27] = Some(self.gpr[27]);
    reg[Riscv64::X28] = Some(self.gpr[28]);
    reg[Riscv64::X29] = Some(self.gpr[29]);
    reg[Riscv64::X30] = Some(self.gpr[30]);
    reg[Riscv64::X31] = Some(self.gpr[31]);
    reg
  }
}

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

  fn set_syscall_result(&mut self, v: &SyscallResult) {
    match v {
      Ok(regs) => {
        self.gpr[16] = 0;
        match regs {
          SyscallOutRegisters::Unit => {}
          SyscallOutRegisters::Single(a) => {
            self.gpr[10] = *a as u64;
          }
          SyscallOutRegisters::Double(a, b) => {
            self.gpr[10] = *a as u64;
            self.gpr[11] = *b as u64;
          }
          SyscallOutRegisters::Triple(a, b, c) => {
            self.gpr[10] = *a as u64;
            self.gpr[11] = *b as u64;
            self.gpr[12] = *c as u64;
          }
          SyscallOutRegisters::Quadruple(a, b, c, d) => {
            self.gpr[10] = *a as u64;
            self.gpr[11] = *b as u64;
            self.gpr[12] = *c as u64;
            self.gpr[13] = *d as u64;
          }
          SyscallOutRegisters::Pentad(a, b, c, d, e) => {
            self.gpr[10] = *a as u64;
            self.gpr[11] = *b as u64;
            self.gpr[12] = *c as u64;
            self.gpr[13] = *d as u64;
            self.gpr[14] = *e as u64;
          }
        }
      }
      Err(e) => {
        self.gpr[16] = *e as u64;
      }
    }
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

