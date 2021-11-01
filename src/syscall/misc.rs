use alloc::boxed::Box;
use super::{Result, SyscallOutRegisters::*};
use common::syscall::error::ERROR_INVARG;

struct ResourceA;
struct ResourceB;
impl Drop for ResourceA {
  fn drop(&mut self) {
    info!("resource a drop")
  }
}
#[inline(never)]
fn make_page_fault() {
  unsafe { (0xdeadbeef0000 as *mut usize).write(0); }
  panic!(); // indicates an exception may happen
}
#[inline(never)]
pub fn null2() -> Result {
  info!("null called");
  let a = Box::new(ResourceA);
  make_page_fault();
  // panic!();
  let b = Box::new(ResourceB);
  Box::leak(a);
  Box::leak(b);
  Ok(Unit)
}

#[inline(never)]
pub fn null() -> Result {
  Ok(Unit)
}

#[inline(never)]
#[inject::count_stmts]
pub fn putc(c: char) -> Result {
  crate::driver::uart::putc(c as u8);
  Ok(Unit)
}

#[inline(never)]
pub fn getc() -> Result {
  match crate::driver::uart::getc() {
    None => Err(common::syscall::error::ERROR_HOLD_ON),
    Some(c) => Ok(Single(c as usize))
  }
}

#[inline(never)]
#[inject::count_stmts]
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
