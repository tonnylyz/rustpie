use rlibc::memcpy;

use crate::arch::page_table::*;
use crate::config::*;
use crate::syscall::*;

pub fn page_fault_handler(va: usize) {
  assert_eq!(va % PAGE_SIZE, 0);
  assert!(va < USER_LIMIT);
  if let Some(pte) = query(va) {
    if !pte.copy_on_write {
      panic!("page_fault_handler: not copy on write");
    }
    let mut va_tmp = STACK_TOP - 2 * PAGE_SIZE;
    loop {
      if let Some(_) = query(va_tmp) {
        va_tmp -= PAGE_SIZE;
      } else {
        break;
      }
    }
    match mem_alloc(0, va_tmp, PTE_W) {
      Ok(_) => {}
      Err(_) => { panic!("page_fault_handler: mem_alloc failed") }
    }
    unsafe {
      memcpy(va_tmp as *mut u8, va as *mut u8, PAGE_SIZE);
    }
    match mem_map(0, va_tmp, 0, va, pte + PTE_W - PTE_COW) {
      Ok(_) => {}
      Err(_) => { panic!("page_fault_handler: mem_map failed") }
    }
    match mem_unmap(0, va_tmp) {
      Ok(_) => {}
      Err(_) => { panic!("page_fault_handler: mem_unmap failed") }
    }
  } else {
    match mem_alloc(0, va, PTE_DEFAULT) {
      Ok(_) => {}
      Err(_) => { panic!("page_fault_handler: mem_alloc failed") }
    }
  }
}

extern "C" {
  fn asm_page_fault_handler() -> !;
}

#[no_mangle]
pub static mut page_fault_handler_stub: usize = 0;

pub fn set_page_fault_handler(handler: usize) {
  unsafe {
    if page_fault_handler_stub == 0 {
      match mem_alloc(0, EXCEPTION_STACK_TOP - PAGE_SIZE, PTE_W) {
        Ok(_) => {}
        Err(_) => { panic!("set_page_fault_handler: mem_alloc failed") }
      }
      match event_handler(0, asm_page_fault_handler as usize, EXCEPTION_STACK_TOP, 0) {
        Ok(_) => {}
        Err(_) => { panic!("set_page_fault_handler: process_set_exception_handler failed") }
      }
    }
    page_fault_handler_stub = handler;
  }
}
