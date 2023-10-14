use rpabi::syscall::error::ERROR_INVARG;

use crate::kernel::print::DebugUart;

use super::{Result, SyscallOutRegisters::*, VOID};

#[inline(never)]
pub fn null() -> Result {
  VOID
}

#[inline(never)]
pub fn putc(c: char) -> Result {
  let mut c = c as u8;
  let uart = crate::board::DEBUG_UART.get().unwrap();
  if c == 127 {
    uart.putc(8);
    uart.putc(b' ');
    c = 8;
  }
  uart.putc(c);
  VOID
}

#[inline(never)]
pub fn getc() -> Result {
  let uart = crate::board::DEBUG_UART.get().unwrap();
  match uart.getc() {
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
