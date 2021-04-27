use rlibc::memcpy;

use crate::arch::page_table::*;
use crate::config::*;
use crate::syscall::*;
use crate::arch::EntryLike;

pub fn page_fault_handler(va: usize) {
  assert_eq!(va % PAGE_SIZE, 0);
  assert!(va < USER_LIMIT);
  if let Some(pte) = query(va) {
    if !pte.copy_on_write() {
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
    mem_alloc(0, va_tmp, Entry::default());
    unsafe {
      memcpy(va_tmp as *mut u8, va as *mut u8, PAGE_SIZE);
    }
    let mut new_attr = pte;
    new_attr.set_writable(true);
    new_attr.set_copy_on_write(false);
    mem_map(0, va_tmp, 0, va, new_attr);
    mem_unmap(0, va_tmp);
  } else {
    mem_alloc(0, va, Entry::default());
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
      mem_alloc(0, EXCEPTION_STACK_TOP - PAGE_SIZE, Entry::default());
      event_handler(0, asm_page_fault_handler as usize, EXCEPTION_STACK_TOP, 0);
    }
    page_fault_handler_stub = handler;
  }
}
