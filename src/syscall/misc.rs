use super::{Result, SyscallOutRegisters::*};

#[inline(never)]
pub fn null() -> Result {
  Ok(Unit)
}

#[inline(never)]
pub fn putc(c: char) -> Result {
  crate::driver::uart::putc(c as u8);
  Ok(Unit)
}
