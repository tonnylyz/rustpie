use alloc::boxed::Box;
use core::time::Duration;
use common::PAGE_SIZE;
use microcall::{thread_alloc, thread_yield, thread_set_status};

pub struct Thread {
  id: usize,
}

unsafe impl Send for Thread {}
unsafe impl Sync for Thread {}

pub type IoResult<T> = core::result::Result<T, ()>; // alias of io::Result

impl Thread {
  pub unsafe fn new(stack: usize, p: Box<dyn FnOnce()>) -> IoResult<Thread> {
    let p = Box::into_raw(box p);
    let stack_size = core::cmp::max(stack, min_stack_size());

    assert_eq!(stack_size % PAGE_SIZE, 0);

    let stack = super::thread_stack::Stack::new();
    let native = thread_alloc(0, thread_start as usize, stack.top(), p as *mut _ as usize);

    extern "C" fn thread_start(main: usize) -> usize {
      unsafe {
        Box::from_raw(main as *mut Box<dyn FnOnce()>)();
      }
      microcall::thread_destroy(0);
      0
    }

    match native {
      Ok(native) => {
        thread_set_status(native, common::thread::THREAD_STATUS_RUNNABLE);
        Ok(Thread { id: native })
      },
      Err(_) => {
        drop(Box::from_raw(p));
        Err(())
      }
    }
  }

  pub fn yield_now() {
    thread_yield()
  }

  pub fn set_name(_name: &'static str) { /* do nothing */ }

  pub fn sleep(_dur: Duration) {
    unimplemented!()
  }

  pub fn join(self) {
    // pthread_join
    unimplemented!()
  }

  pub fn id(&self) -> usize {
    self.id
  }

  pub fn into_id(self) -> usize {
    let id = self.id;
    core::mem::forget(self);
    id
  }
}

impl Drop for Thread {
  fn drop(&mut self) {
    // pthread_detach
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
