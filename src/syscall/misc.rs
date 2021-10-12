use super::{Result, SyscallOutRegisters::*};
use common::syscall::error::ERROR_INVARG;

#[inline(never)]
pub fn null() -> Result {
  Ok(Unit)
}

#[inline(never)]
pub fn putc(c: char) -> Result {
  crate::driver::uart::putc(c as u8);
  Ok(Unit)
}

#[inline(never)]
pub fn set_exception_handler(handler: usize) -> Result {
  let t = super::current_thread()?;
  match t.address_space() {
    None => Err(ERROR_INVARG),
    Some(a) => {
      a.set_exception_handler(Some(handler));
      Ok(Unit)
    }
  }
}
