global_asm!(include_str!("start.S"));

// Workaround for abort symbol not found
#[no_mangle]
pub extern "C" fn abort() {
  panic!("abort");
}

#[no_mangle]
pub unsafe fn core_stack() -> usize {
  crate::lib::core::stack().top()
}
