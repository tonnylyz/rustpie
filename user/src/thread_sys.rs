use alloc::boxed::Box;
use crate::config::PAGE_SIZE;
use core::time::Duration;
use crate::mem::valloc;
use crate::microcall::*;

pub const DEFAULT_MIN_STACK_SIZE: usize = 2 * 1024 * 1024;

pub struct Thread {
  id: u16,
}

unsafe impl Send for Thread {}
unsafe impl Sync for Thread {}

pub type IoResult<T> = core::result::Result<T, ()>; // alias of io::Result

impl Thread {
  pub unsafe fn new(stack: usize, p: Box<dyn FnOnce()>) -> IoResult<Thread> {
    let p = Box::into_raw(box p);
    let stack_size = core::cmp::max(stack, min_stack_size());

    assert_eq!(stack_size % PAGE_SIZE, 0);
    let sp = valloc(stack_size / PAGE_SIZE);

    let native = thread_alloc(thread_start as usize, sp as usize + stack_size, p as *mut _ as usize);

    extern "C" fn thread_start(main: usize) -> usize {
      unsafe {
        Box::from_raw(main as *mut Box<dyn FnOnce()>)();
      }
      0
    }

    return if native == u16::MAX {
      drop(Box::from_raw(p));
      Err(())
    } else {
      Ok(Thread { id: native })
    }
  }

  pub fn yield_now() {
    thread_yield()
  }

  pub fn set_name(name: &'static str) { /* do nothing */ }

  pub fn sleep(dur: Duration) {
    unimplemented!()
  }

  pub fn join(self) {
    // pthread_join
    unimplemented!()
  }

  pub fn id(&self) -> u16 {
    self.id
  }

  pub fn into_id(self) -> u16 {
    let id = self.id;
    core::mem::forget(self);
    id
  }
}

impl Drop for Thread {
  fn drop(&mut self) {
    // pthread_detach
    unimplemented!()
  }
}

pub mod guard {
  use core::ops::Range;
  pub type Guard = Range<usize>;

  pub unsafe fn current() -> Option<Guard> {
    None
  }
  pub unsafe fn init() -> Option<Guard> {
    None
  }
}

fn min_stack_size() -> usize {
  PAGE_SIZE
}
