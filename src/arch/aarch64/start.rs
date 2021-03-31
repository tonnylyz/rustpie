global_asm!(include_str!("start.S"));

#[no_mangle]
pub unsafe fn core_stack() -> usize {
  crate::lib::stack::stack().top()
}
