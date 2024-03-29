use crate::{arch::PAGE_SIZE, MAX_CPU_NUMBER};

const STACK_PAGE_NUM: usize = 64;

#[repr(align(4096))]
pub struct Stack {
  stack: [u8; PAGE_SIZE * STACK_PAGE_NUM],
}

impl Stack {
  pub fn top(&self) -> usize {
    (&self.stack as *const _ as usize) + PAGE_SIZE * STACK_PAGE_NUM
  }
}

const STACK: Stack = Stack {
  stack: [0; PAGE_SIZE * STACK_PAGE_NUM],
};

#[link_section = ".stack"]
static STACKS: [Stack; MAX_CPU_NUMBER] = [STACK; MAX_CPU_NUMBER];

#[no_mangle]
pub fn stack_of_core(core_id: usize) -> usize {
  STACKS[core_id].top()
}
