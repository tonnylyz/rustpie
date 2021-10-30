pub mod misc;
pub mod thread;
pub mod mm;
pub mod address_space;
pub mod event;
pub mod ipc;
pub mod server;

use core::fmt::{Display, Formatter};

pub type Error = usize;

#[derive(Debug)]
pub enum SyscallOutRegisters {
  Unit,
  Single(usize),
  Double(usize, usize),
  Triple(usize, usize, usize),
  Quadruple(usize, usize, usize, usize),
  Pentad(usize, usize, usize, usize, usize),
}

use SyscallOutRegisters::*;
use crate::lib::address_space::AddressSpace;
use common::syscall::error::*;
use crate::lib::thread::Thread;
use crate::lib::cpu::cpu;

impl Display for SyscallOutRegisters {
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    match self {
      Unit => { write!(f, "()")? }
      Single(a) => { write!(f, "({:x})", a)? }
      Double(a, b) => { write!(f, "({:x}, {:x})", a, b)? }
      Triple(a, b, c) => { write!(f, "({:x}, {:x}, {:x})", a, b, c)? }
      Quadruple(a, b, c, d) => { write!(f, "({:x}, {:x}, {:x}, {:x})", a, b, c, d)? }
      Pentad(a, b, c, d, e) => { write!(f, "({:x}, {:x}, {:x}, {:x}, {:x})", a, b, c, d, e)? }
    }
    Ok(())
  }
}

pub type Result = core::result::Result<SyscallOutRegisters, Error>;

fn lookup_as(asid: u16) -> core::result::Result<AddressSpace, Error> {
  let a = if asid == 0 {
    current_thread()?.address_space()
  } else {
    crate::lib::address_space::address_space_lookup(asid)
  };
  a.ok_or(ERROR_INVARG)
}

fn current_thread() -> core::result::Result<Thread, Error> {
  match cpu().running_thread() {
    None => Err(ERROR_INTERNAL),
    Some(t) => Ok(t),
  }
}
