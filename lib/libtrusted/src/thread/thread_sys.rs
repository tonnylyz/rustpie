use alloc::boxed::Box;

use common::PAGE_SIZE;

use microcall::{thread_alloc, thread_set_status};

use crate::mm::{virtual_alloc, virtual_free};

pub struct Thread {
  id: usize,
  stack_btm: usize,
  stack_size_in_page: usize,
}

unsafe impl Send for Thread {}

unsafe impl Sync for Thread {}

pub type IoResult<T> = core::result::Result<T, ()>; // alias of io::Result

const THREAD_STACK_PAGE_NUM: usize = 48;

impl Thread {
  pub unsafe fn new(p: Box<dyn FnOnce()>) -> IoResult<Thread> {
    let p = Box::into_raw(Box::new(p));

    let stack = virtual_alloc(THREAD_STACK_PAGE_NUM, true).unwrap();
    let stack_top = stack + THREAD_STACK_PAGE_NUM * PAGE_SIZE;
    let native = thread_alloc(0, thread_start as usize, stack_top, p as *mut _ as usize);

    extern "C" fn thread_start(main: usize) -> usize {
      unsafe {
        Box::from_raw(main as *mut Box<dyn FnOnce()>)();
      }
      let _ = microcall::thread_destroy(0);
      0
    }

    match native {
      Ok(native) => {
        let _ = thread_set_status(native, common::thread::THREAD_STATUS_RUNNABLE);
        Ok(Thread {
          id: native,
          stack_btm: stack,
          stack_size_in_page: THREAD_STACK_PAGE_NUM,
        })
      }
      Err(_) => {
        drop(Box::from_raw(p));
        Err(())
      }
    }
  }

  pub fn join(self) {
    loop {
      if let Ok(_) = microcall::event_wait(common::event::EVENT_THREAD_EXIT, self.id) {
        virtual_free(self.stack_btm, self.stack_size_in_page);
        break;
      } else {
        microcall::thread_yield();
      }
    }
  }

  pub fn id(&self) -> usize {
    self.id
  }
}

impl Drop for Thread {
  fn drop(&mut self) {
    // pthread_detach
    info!("thread drop, stack may leak");
  }
}
