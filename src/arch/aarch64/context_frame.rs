use core::fmt::Formatter;
use crate::ContextFrameTrait;
use crate::lib::syscall::{Result as SyscallResult, SyscallOutRegisters};

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Aarch64ContextFrame {
  gpr: [u64; 31],
  spsr: u64,
  elr: u64,
  sp: u64,
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
