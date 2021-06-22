mod thread_sys;
mod thread_parker;
mod thread_stack;

use alloc::boxed::Box;
use alloc::string::String;
use alloc::sync::Arc;
use core::any::Any;
use core::cell::UnsafeCell;
use core::num::NonZeroU64;
use core::time::Duration;

use common::PAGE_SIZE;
use thread_parker::Parker;
use thread_sys as imp;

////////////////////////////////////////////////////////////////////////////////
// Builder
////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct Builder {
  // A name for the thread-to-be, for identification in panic messages
  name: Option<String>,
  // The size of the stack for the spawned thread in bytes
  stack_size: Option<usize>,
}

impl Builder {
  pub fn new() -> Builder {
    Builder { name: None, stack_size: None }
  }
  pub fn name(mut self, name: String) -> Builder {
    self.name = Some(name);
    self
  }
  pub fn stack_size(mut self, size: usize) -> Builder {
    self.stack_size = Some(size);
    self
  }
  pub fn spawn<F, T>(self, f: F) -> core::result::Result<JoinHandle<T>, ()>
    where
      F: FnOnce() -> T,
      F: Send + 'static,
      T: Send + 'static,
  {
    unsafe { self.spawn_unchecked(f) }
  }
  pub unsafe fn spawn_unchecked<'a, F, T>(self, f: F) -> core::result::Result<JoinHandle<T>, ()>
    where
      F: FnOnce() -> T,
      F: Send + 'a,
      T: Send + 'a,
  {
    let Builder { name, stack_size } = self;

    let stack_size = stack_size.unwrap_or_else(/*thread::min_stack*/|| PAGE_SIZE);

    let my_thread = Thread::new(name);
    // let their_thread = my_thread.clone();

    let my_packet: Arc<UnsafeCell<Option<Result<T>>>> = Arc::new(UnsafeCell::new(None));
    let their_packet = my_packet.clone();

    // let output_capture = crate::io::set_output_capture(None);
    // crate::io::set_output_capture(output_capture.clone());

    let main = move || {
      // crate::io::set_output_capture(output_capture);

      // SAFETY: the stack guard passed is the one for the current thread.
      // This means the current thread's stack and the new thread's stack
      // are properly set and protected from each other.
      // thread_info::set(unsafe { imp::guard::current() }, their_thread);
      // let try_result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
      // crate::sys_common::backtrace::__rust_begin_short_backtrace(f)
      // }));
      let try_result = Ok(f());

      // SAFETY: `their_packet` as been built just above and moved by the
      // closure (it is an Arc<...>) and `my_packet` will be stored in the
      // same `JoinInner` as this closure meaning the mutation will be
      // safe (not modify it and affect a value far away).
      unsafe { *their_packet.get() = Some(try_result) };
    };

    Ok(JoinHandle(JoinInner {
      // SAFETY:
      //
      // `imp::Thread::new` takes a closure with a `'static` lifetime, since it's passed
      // through FFI or otherwise used with low-level threading primitives that have no
      // notion of or way to enforce lifetimes.
      //
      // As mentioned in the `Safety` section of this function's documentation, the caller of
      // this function needs to guarantee that the passed-in lifetime is sufficiently long
      // for the lifetime of the thread.
      //
      // Similarly, the `sys` implementation must guarantee that no references to the closure
      // exist after the thread has terminated, which is signaled by `Thread::join`
      // returning.
      native: Some(imp::Thread::new(
        stack_size,
        core::mem::transmute::<Box<dyn FnOnce() + 'a>, Box<dyn FnOnce() + 'static>>(Box::new(main)))?),
      thread: my_thread,
      packet: Packet(my_packet),
    }))
  }
}

////////////////////////////////////////////////////////////////////////////////
// Free functions
////////////////////////////////////////////////////////////////////////////////

pub fn spawn<F, T>(f: F) -> JoinHandle<T>
  where
    F: FnOnce() -> T,
    F: Send + 'static,
    T: Send + 'static,
{
  Builder::new().spawn(f).expect("failed to spawn thread")
}

pub fn current() -> Thread {
  unimplemented!()
}

pub fn yield_now() {
  imp::Thread::yield_now()
}

/// Determines whether the current thread is unwinding because of panic.
pub fn panicking() -> bool {
  unimplemented!()
}

pub fn sleep_ms(ms: u32) {
  sleep(Duration::from_millis(ms as u64))
}

pub fn sleep(dur: Duration) {
  imp::Thread::sleep(dur)
}

pub fn park() {
  // SAFETY: park_timeout is called on the parker owned by this thread.
  unsafe {
    current().inner.parker.park();
  }
}

pub fn park_timeout_ms(ms: u32) {
  park_timeout(Duration::from_millis(ms as u64))
}

pub fn park_timeout(dur: Duration) {
  // SAFETY: park_timeout is called on the parker owned by this thread.
  unsafe {
    current().inner.parker.park_timeout(dur);
  }
}

////////////////////////////////////////////////////////////////////////////////
// ThreadId
////////////////////////////////////////////////////////////////////////////////

#[derive(Eq, PartialEq, Clone, Copy, Hash, Debug)]
pub struct ThreadId(NonZeroU64);

impl ThreadId {
  fn new() -> ThreadId {
    static GUARD: spin::mutex::Mutex<()> = spin::mutex::Mutex::new(());
    static mut COUNTER: u64 = 1;
    unsafe {
      let _guard = GUARD.lock();
      if COUNTER == u64::MAX {
        panic!("failed to generate unique thread ID: bitspace exhausted");
      }
      let id = COUNTER;
      COUNTER += 1;
      drop(_guard);
      ThreadId(NonZeroU64::new(id).unwrap())
    }
  }
}


////////////////////////////////////////////////////////////////////////////////
// Thread
////////////////////////////////////////////////////////////////////////////////

struct Inner {
  //name: Option<String>,
  id: ThreadId,
  parker: Parker,
}

#[derive(Clone)]
pub struct Thread {
  inner: Arc<Inner>,
}

impl Thread {
  pub fn new(_name: Option<String>) -> Thread {
    Thread {
      inner: Arc::new(Inner {
        //name,
        id: ThreadId::new(),
        parker: Parker::new(),
      })
    }
  }
  pub fn unpark(&self) {
    self.inner.parker.unpark();
  }
  pub fn id(&self) -> ThreadId {
    self.inner.id
  }
  pub fn name(&self) -> Option<&str> {
    None
  }
}


////////////////////////////////////////////////////////////////////////////////
// JoinHandle
////////////////////////////////////////////////////////////////////////////////

pub type Result<T> = core::result::Result<T, Box<dyn Any + Send + 'static>>;

struct Packet<T>(Arc<UnsafeCell<Option<Result<T>>>>);

unsafe impl<T: Send> Send for Packet<T> {}

unsafe impl<T: Sync> Sync for Packet<T> {}

struct JoinInner<T> {
  native: Option<imp::Thread>,
  thread: Thread,
  packet: Packet<T>,
}

impl<T> JoinInner<T> {
  fn join(&mut self) -> Result<T> {
    self.native.take().unwrap().join();
    unsafe { (*self.packet.0.get()).take().unwrap() }
  }
}

pub struct JoinHandle<T>(JoinInner<T>);

unsafe impl<T> Send for JoinHandle<T> {}

unsafe impl<T> Sync for JoinHandle<T> {}

impl<T> JoinHandle<T> {
  pub fn thread(&self) -> &Thread {
    &self.0.thread
  }
  pub fn native(&self) -> u16 {
    self.0.native.as_ref().unwrap().id()
  }
  pub fn join(mut self) -> Result<T> {
    self.0.join()
  }
}
