use core::ops::Range;
use spin::Mutex;
use common::PAGE_SIZE;
use crate::mm::{Entry, PageAttribute, default_page_attribute};
use microcall::mem_alloc;

const THREAD_STACK_MAX: usize = 48 * PAGE_SIZE;

const THREAD_STACK_VALLOC_RANGE: Range<usize> = 0x1000_0000..0x2000_0000;

static POINTER: Mutex<usize> = Mutex::new(0);

pub struct Stack {
  range: Range<usize>,
}

impl Stack {
  pub fn new() -> Self {
    let mut ptr = POINTER.lock();
    if *ptr == 0 {
      *ptr = THREAD_STACK_VALLOC_RANGE.start;
    }
    let r = Stack {
      range: *ptr..(*ptr + THREAD_STACK_MAX)
    };
    for i in r.range.clone().step_by(PAGE_SIZE) {
      mem_alloc(0, i, default_page_attribute());
    }
    *ptr += THREAD_STACK_MAX;
    r
  }

  pub fn top(&self) -> usize {
    self.range.end
  }
}

