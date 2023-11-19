use core::fmt::Formatter;
use core::panic;

use crate::ContextFrameTrait;
use crate::syscall::SyscallOutRegisters;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct X64ContextFrame {
  rax: u64,
  rbx: u64,
  rcx: u64,
  rdx: u64,
  rsi: u64,
  rdi: u64,
  rbp: u64,
  r8: u64,
  r9: u64,
  r10: u64,
  r11: u64,
  /* r12-r15 callee saved */
  r12: u64,
  r13: u64,
  r14: u64,
  r15: u64,
  /* iret pop order */
  rip: u64,
  cs: u64,
  rflags: u64,
  rsp: u64,
  ss: u64,
}

impl core::fmt::Display for X64ContextFrame {
  fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), core::fmt::Error> {
    write!(f, "rax: {:016x}  ", self.rax)?;
    write!(f, "rbx: {:016x}  ", self.rbx)?;
    writeln!(f)?;
    write!(f, "rcx: {:016x}  ", self.rcx)?;
    write!(f, "rdx: {:016x}  ", self.rdx)?;
    writeln!(f)?;
    write!(f, "rsi: {:016x}  ", self.rsi)?;
    write!(f, "rdi: {:016x}  ", self.rdi)?;
    writeln!(f)?;
    write!(f, "rbp: {:016x}  ", self.rbp)?;
    write!(f, "rsp: {:016x}  ", self.rsp)?;
    writeln!(f)?;
    write!(f, "r8:  {:016x}  ", self.r8)?;
    write!(f, "r9:  {:016x}  ", self.r9)?;
    writeln!(f)?;
    write!(f, "r10: {:016x}  ", self.r10)?;
    write!(f, "r11: {:016x}  ", self.r11)?;
    writeln!(f)?;
    write!(f, "r12: {:016x}  ", self.r12)?;
    write!(f, "r13: {:016x}  ", self.r13)?;
    writeln!(f)?;
    write!(f, "r14: {:016x}  ", self.r14)?;
    write!(f, "r15: {:016x}  ", self.r15)?;
    writeln!(f)?;
    write!(f, "flag:{:016x}  ", self.rflags)?;
    write!(f, "rip: {:016x}  ", self.rip)?;
    writeln!(f)?;
    write!(f, "cs:  {:016x}  ", self.cs)?;
    write!(f, "ss:  {:016x}  ", self.ss)?;
    writeln!(f)?;
    Ok(())
  }
}

impl ContextFrameTrait for X64ContextFrame {
  fn new(pc: usize, sp: usize, arg: usize, privileged: bool) -> Self {
    use x86_64::registers::rflags::RFlags;
    let mut flags = RFlags::from_bits_retain(0x2); // EFLAGS (bit 1)
    flags |= RFlags::INTERRUPT_FLAG;
    if privileged {
      flags |= RFlags::IOPL_LOW;
    }

    let mut r = X64ContextFrame {
      rip: pc as u64,
        rax: 0,
        rbx: 0,
        rcx: 0,
        rdx: 0,
        rsi: 0,
        rdi: 0,
        rbp: 0,
        rsp: sp as u64,
        r8:  0,
        r9:  0,
        r10: 0,
        r11: 0,
        r12: 0,
        r13: 0,
        r14: 0,
        r15: 0,
        rflags: flags.bits(),
        cs: if privileged { 8 } else { 27 },
        ss: if privileged { 16 } else { 35 },
    };
    r.set_argument(arg);
    r
  }

  fn syscall_argument(&self, i: usize) -> usize {
    (match i {
      0 => self.rdi,
      1 => self.rsi,
      2 => self.rdx,
      3 => self.r10,
      4 => self.r8,
      5 => self.r9,
      _ => panic!("x64 syscall only supports 6 arguments")
    }) as usize
  }

  fn syscall_number(&self) -> usize {
    self.rax as usize
  }

  fn set_syscall_result(&mut self, v: &SyscallOutRegisters, err: usize) {
    self.rax = err as u64;
    match v {
      SyscallOutRegisters::Unit => {}
      SyscallOutRegisters::Single(a) => {
        self.rdi = *a as u64;
      }
      SyscallOutRegisters::Double(a, b) => {
        self.rdi = *a as u64;
        self.rsi = *b as u64;
      }
      SyscallOutRegisters::Triple(a, b, c) => {
        self.rdi = *a as u64;
        self.rsi = *b as u64;
        self.rdx = *c as u64;
      }
      SyscallOutRegisters::Quadruple(a, b, c, d) => {
        self.rdi = *a as u64;
        self.rsi = *b as u64;
        self.rdx = *c as u64;
        self.r10 = *d as u64;
      }
      SyscallOutRegisters::Pentad(a, b, c, d, e) => {
        self.rdi = *a as u64;
        self.rsi = *b as u64;
        self.rdx = *c as u64;
        self.r10 = *d as u64;
        self.r8 = *e as u64;
      }
    }
  }

  fn exception_pc(&self) -> usize {
    self.rip as usize
  }

  fn set_exception_pc(&mut self, pc: usize) {
    self.rip = pc as u64;
  }

  fn stack_pointer(&self) -> usize {
    self.rsp as usize
  }

  fn set_stack_pointer(&mut self, sp: usize) {
    self.rsp = sp as u64;
  }

  fn set_argument(&mut self, arg: usize) {
    self.rdi = arg as u64;
  }

  fn gpr(&self, index: usize) -> usize {
    panic!()
  }
}

impl X64ContextFrame {
  pub fn frame_pointer(&self) -> usize {
    self.rbp as usize
  }
}
