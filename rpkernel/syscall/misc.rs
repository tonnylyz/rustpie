use rpabi::syscall::error::ERROR_INVARG;

use super::{Result, SyscallOutRegisters::*, VOID};

#[inline(never)]
pub fn null() -> Result {
  VOID
}

#[inline(never)]
pub fn putc(c: char) -> Result {
  let mut c = c as u8;
  if c == 127 {
    crate::driver::uart::putc(8);
    crate::driver::uart::putc(b' ');
    c = 8;
  }
  crate::driver::uart::putc(c);
  VOID
}

#[inline(never)]
pub fn getc() -> Result {
  match crate::driver::uart::getc() {
    None => Err(rpabi::syscall::error::ERROR_HOLD_ON),
    Some(c) => Ok((Single(c as usize), false))
  }
}

#[inline(never)]
pub fn set_exception_handler(handler: usize) -> Result {
  let t = super::current_thread()?;
  match t.address_space() {
    None => Err(ERROR_INVARG),
    Some(a) => {
      a.set_exception_handler(Some(handler));
      VOID
    }
  }
}
