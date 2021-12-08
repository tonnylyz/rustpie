use alloc::boxed::Box;

use common::syscall::error::ERROR_INVARG;

use super::{Result, SyscallOutRegisters::*};

#[allow(dead_code)]
struct ResourceA;

#[allow(dead_code)]
struct ResourceB;

impl Drop for ResourceA {
  fn drop(&mut self) {
    info!("resource a drop")
  }
}

#[inline(never)]
#[allow(dead_code)]
fn make_page_fault() {
  unsafe { (0xdeadbeef0000 as *mut usize).write(0); };
  panic!()
}

#[inline(never)]
#[allow(dead_code)]
#[allow(unreachable_code)]
#[allow(unused_variables)]
pub fn null(dummy: usize) -> Result {
  match dummy {
    0 => {
      Ok(Unit) // normal null call
    }
    1 => {
      info!("null called - kernel panic");
      let a = Box::new(ResourceA);
      panic!();
      let b = Box::new(ResourceB);
      Box::leak(a);
      Box::leak(b);
      Ok(Unit)
    }
    2 => {
      info!("null called - kernel page fault");
      let a = Box::new(ResourceA);
      make_page_fault();
      let b = Box::new(ResourceB);
      Box::leak(a);
      Box::leak(b);
      Ok(Unit)
    }
    _ => Ok(Unit)
  }
}

#[inline(never)]
#[inject::count_stmts]
#[inject::panic_inject]
#[inject::page_fault_inject]
pub fn putc(c: char) -> Result {
  let mut c = c as u8;
  if c == 127 {
    crate::driver::uart::putc(8);
    crate::driver::uart::putc(b' ');
    c = 8;
  }
  crate::driver::uart::putc(c);
  Ok(Unit)
}

#[inline(never)]
#[inject::count_stmts]
#[inject::panic_inject]
#[inject::page_fault_inject]
pub fn getc() -> Result {
  match crate::driver::uart::getc() {
    None => Err(common::syscall::error::ERROR_HOLD_ON),
    Some(c) => Ok(Single(c as usize))
  }
}

#[inline(never)]
#[inject::count_stmts]
#[inject::panic_inject]
#[inject::page_fault_inject]
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
