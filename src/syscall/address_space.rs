use rpabi::syscall::error::*;

use crate::lib::thread::Tid;

use super::{Result, SyscallOutRegisters::*};

#[inline(never)]
pub fn get_asid(tid: Tid) -> Result {
  if tid == 0 {
    match crate::lib::cpu::cpu().address_space() {
      None => Err(ERROR_INTERNAL),
      Some(a) => Ok(Single(a.asid() as usize)),
    }
  } else {
    match crate::lib::thread::thread_lookup(tid) {
      None => Err(ERROR_INVARG),
      Some(t) => match t.address_space() {
        None => Err(ERROR_INVARG),
        Some(a) => Ok(Single(a.asid() as usize)),
      },
    }
  }
}

#[inline(never)]
pub fn address_space_alloc() -> Result {
  let a = crate::lib::address_space::address_space_alloc()?;
  Ok(Single(a.asid() as usize))
}

#[inline(never)]
pub fn address_space_destroy(asid: u16) -> Result {
  let a = super::lookup_as(asid)?;
  crate::lib::address_space::address_space_destroy(a);
  Ok(Unit)
}
