use core::fmt::Formatter;
use crate::ContextFrameTrait;
use crate::syscall::{Result as SyscallResult, SyscallOutRegisters};
use unwind::registers::Registers;
use unwind::arch::Aarch64;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Aarch64ContextFrame {
  gpr: [u64; 31],
  spsr: u64,
  elr: u64,
  sp: u64,
}

impl Into<Registers> for Aarch64ContextFrame {
  fn into(self) -> Registers {
    let mut reg = Registers::default();
    reg[Aarch64::X0] = Some(self.gpr[0]);
    reg[Aarch64::X1] = Some(self.gpr[1]);
    reg[Aarch64::X2] = Some(self.gpr[2]);
    reg[Aarch64::X3] = Some(self.gpr[3]);
    reg[Aarch64::X4] = Some(self.gpr[4]);
    reg[Aarch64::X5] = Some(self.gpr[5]);
    reg[Aarch64::X6] = Some(self.gpr[6]);
    reg[Aarch64::X7] = Some(self.gpr[7]);
    reg[Aarch64::X8] = Some(self.gpr[8]);
    reg[Aarch64::X9] = Some(self.gpr[9]);
    reg[Aarch64::X10] = Some(self.gpr[10]);
    reg[Aarch64::X11] = Some(self.gpr[11]);
    reg[Aarch64::X12] = Some(self.gpr[12]);
    reg[Aarch64::X13] = Some(self.gpr[13]);
    reg[Aarch64::X14] = Some(self.gpr[14]);
    reg[Aarch64::X15] = Some(self.gpr[15]);
    reg[Aarch64::X16] = Some(self.gpr[16]);
    reg[Aarch64::X17] = Some(self.gpr[17]);
    reg[Aarch64::X18] = Some(self.gpr[18]);
    reg[Aarch64::X19] = Some(self.gpr[19]);
    reg[Aarch64::X20] = Some(self.gpr[20]);
    reg[Aarch64::X21] = Some(self.gpr[21]);
    reg[Aarch64::X22] = Some(self.gpr[22]);
    reg[Aarch64::X23] = Some(self.gpr[23]);
    reg[Aarch64::X24] = Some(self.gpr[24]);
    reg[Aarch64::X25] = Some(self.gpr[25]);
    reg[Aarch64::X26] = Some(self.gpr[26]);
    reg[Aarch64::X27] = Some(self.gpr[27]);
    reg[Aarch64::X28] = Some(self.gpr[28]);
    reg[Aarch64::X29] = Some(self.gpr[29]);
    reg[Aarch64::X30] = Some(self.gpr[30]);
    reg[Aarch64::SP] = Some(self.sp);
    reg
  }
}

impl core::fmt::Display for Aarch64ContextFrame {
  fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), core::fmt::Error> {
    for i in 0..31 {
      write!(f, "x{:02}: {:016x}   ", i, self.gpr[i])?;
      if (i + 1) % 2 == 0 {
        write!(f, "\n")?;
      }
    }
    writeln!(f, "spsr:{:016x}", self.spsr)?;
    write!(f, "elr: {:016x}", self.elr)?;
    writeln!(f, "   sp:  {:016x}", self.sp)?;
    Ok(())
  }
}

impl ContextFrameTrait for Aarch64ContextFrame {
  fn new(pc: usize, sp: usize, arg: usize, privileged: bool) -> Self {
    use cortex_a::registers::*;
    let mut r = Aarch64ContextFrame {
      gpr: [0; 31],
      spsr: (
        if privileged { SPSR_EL1::M::EL1t } else { SPSR_EL1::M::EL0t }
          + SPSR_EL1::I::Unmasked + SPSR_EL1::F::Masked).value as u64,
      elr: pc as u64,
      sp: sp as u64,
    };
    r.set_argument(arg);
    r
  }

  fn syscall_argument(&self, i: usize) -> usize {
    const AARCH64_SYSCALL_ARG_LIMIT: usize = 8;
    assert!(i < AARCH64_SYSCALL_ARG_LIMIT);
    // x0 ~ x7
    self.gpr[i] as usize
  }

  fn syscall_number(&self) -> usize {
    // x8
    self.gpr[8] as usize
  }

  fn set_syscall_result(&mut self, v: &SyscallResult) {
    match v {
      Ok(regs) => {
        self.gpr[7] = 0;
        match regs {
          SyscallOutRegisters::Unit => {}
          SyscallOutRegisters::Single(a) => {
            self.gpr[0] = *a as u64;
          }
          SyscallOutRegisters::Double(a, b) => {
            self.gpr[0] = *a as u64;
            self.gpr[1] = *b as u64;
          }
          SyscallOutRegisters::Triple(a, b, c) => {
            self.gpr[0] = *a as u64;
            self.gpr[1] = *b as u64;
            self.gpr[2] = *c as u64;
          }
          SyscallOutRegisters::Quadruple(a, b, c, d) => {
            self.gpr[0] = *a as u64;
            self.gpr[1] = *b as u64;
            self.gpr[2] = *c as u64;
            self.gpr[3] = *d as u64;
          }
          SyscallOutRegisters::Pentad(a, b, c, d, e) => {
            self.gpr[0] = *a as u64;
            self.gpr[1] = *b as u64;
            self.gpr[2] = *c as u64;
            self.gpr[3] = *d as u64;
            self.gpr[4] = *e as u64;
          }
        }
      }
      Err(e) => {
        self.gpr[7] = *e as u64;
      }
    }
  }

  fn exception_pc(&self) -> usize {
    self.elr as usize
  }

  fn set_exception_pc(&mut self, pc: usize) {
    self.elr = pc as u64;
  }

  fn stack_pointer(&self) -> usize {
    self.sp as usize
  }

  fn set_stack_pointer(&mut self, sp: usize) {
    self.sp = sp as u64;
  }

  fn set_argument(&mut self, arg: usize) {
    self.gpr[0] = arg as u64;
  }

  fn gpr(&self, index: usize) -> usize {
    self.gpr[index] as usize
  }
}
