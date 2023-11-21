use rpabi::PAGE_SIZE;
use rpabi::syscall::error::*;

use rpabi::syscall::mm::EntryAttribute;
use crate::util::round_down;

use super::{Result, VOID};

#[inline(never)]
pub fn mem_alloc(asid: u16, va: usize, attr: usize) -> Result {
  let va = round_down(va, PAGE_SIZE);
  let a = super::lookup_as(asid)?;
  let frame = crate::mm::page_pool::page_alloc().map_err(|_| ERROR_OOM)?;
  frame.zero();
  let attr = EntryAttribute::from(attr).filter();
  let uf = crate::mm::Frame::from(frame);
  a.page_table().insert_page(va, uf, attr).map_err(|_| ERROR_INTERNAL)?;
  VOID
}

#[inline(never)]
pub fn mem_map(src_asid: u16, src_va: usize, dst_asid: u16, dst_va: usize, attr: usize) -> Result {
  let src_va = round_down(src_va, PAGE_SIZE);
  let dst_va = round_down(dst_va, PAGE_SIZE);
  let src_as = super::lookup_as(src_asid)?;
  let dst_as = super::lookup_as(dst_asid)?;
  let attr = EntryAttribute::from(attr).filter();
  let pt = src_as.page_table();
  if let Some(uf) = pt.lookup_user_page(src_va) {
    dst_as.page_table().insert_page(dst_va, uf, attr).map_err(|_| ERROR_INTERNAL)?;
    VOID
  } else {
    Err(ERROR_MEM_NOT_MAP)
  }
}

#[inline(never)]
pub fn mem_unmap(asid: u16, va: usize) -> Result {
  let va = round_down(va, PAGE_SIZE);
  let a = super::lookup_as(asid)?;
  a.page_table().remove_page(va).map_err(|_| ERROR_INTERNAL)?;
  VOID
}
