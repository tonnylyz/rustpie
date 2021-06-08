use common::*;
use super::EntryLike;
use microcall::event_handler;

#[no_mangle]
pub fn page_fault_handler(va: usize) {
  assert_eq!(va % PAGE_SIZE, 0);
  assert!(va < CONFIG_USER_LIMIT);
  if let Some(_) = super::query(va) {
    panic!("Trusted userspace unhandled page fault.");
  } else {
    // TODO: check with thread_stack, allocate page passively
    microcall::mem_alloc(0, va, super::Entry::default().attribute());
  }
}

extern "C" {
  fn asm_page_fault_handler() -> !;
}

/// Setup page fault handler per thread
pub fn init() {
  let exception_stack = super::valloc(1);
  event_handler(
    0,
    asm_page_fault_handler as usize,
    exception_stack as usize + PAGE_SIZE,
    0
  );
}
