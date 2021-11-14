use super::{Result, SyscallOutRegisters::*};
use crate::util::round_down;
use common::PAGE_SIZE;
use common::syscall::error::*;
use crate::arch::ArchPageTableEntry;
use crate::lib::traits::ArchPageTableEntryTrait;
use crate::mm::page_table::{Entry, PageTableTrait, PageTableEntryAttrTrait};

#[inline(never)]
#[inject::count_stmts]
#[inject::panic_inject]
#[inject::page_fault_inject]
pub fn mem_alloc(asid: u16, va: usize, attr: usize) -> Result {
  let va = round_down(va, PAGE_SIZE);
  let a = super::lookup_as(asid)?;
  let frame = crate::mm::page_pool::page_alloc().map_err(|_| ERROR_OOM)?;
  frame.zero();
  let attr = Entry::from(ArchPageTableEntry::from_pte(attr)).attribute().filter();
  let uf = crate::mm::Frame::from(frame);
  a.page_table().insert_page(va, uf, attr).map_err(|_| ERROR_INTERNAL)?;
  Ok(Unit)
}

#[inline(never)]
#[inject::count_stmts]
#[inject::panic_inject]
#[inject::page_fault_inject]
pub fn mem_map(src_asid: u16, src_va: usize, dst_asid: u16, dst_va: usize, attr: usize) -> Result {
  let src_va = round_down(src_va, PAGE_SIZE);
  let dst_va = round_down(dst_va, PAGE_SIZE);
  let src_as = super::lookup_as(src_asid)?;
  let dst_as = super::lookup_as(dst_asid)?;
  let attr = Entry::from(ArchPageTableEntry::from_pte(attr)).attribute().filter();
  if let Some(uf) = src_as.page_table().lookup_user_page(src_va) {
    dst_as.page_table().insert_page(dst_va, uf, attr).map_err(|_| ERROR_INTERNAL)?;
    Ok(Unit)
  } else {
    Err(ERROR_MEM_NOT_MAP)
  }
}

#[inline(never)]
#[inject::count_stmts]
#[inject::panic_inject]
#[inject::page_fault_inject]
pub fn mem_unmap(asid: u16, va: usize) -> Result {
  let va = round_down(va, PAGE_SIZE);
  let a = super::lookup_as(asid)?;
  a.page_table().remove_page(va).map_err(|_| ERROR_INTERNAL)?;
  Ok(Unit)
}
