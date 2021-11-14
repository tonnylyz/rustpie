use alloc::boxed::Box;
use alloc::sync::Arc;
use core::any::Any;
use core::cell::UnsafeCell;
use core::num::NonZeroU64;

use thread_sys as imp;

mod thread_sys;

////////////////////////////////////////////////////////////////////////////////
// Builder
////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct Builder;

impl Builder {
  pub fn new() -> Builder {
    Builder
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
    let my_thread = Thread::new();

    let my_packet: Arc<UnsafeCell<Option<Result<T>>>> = Arc::new(UnsafeCell::new(None));
    let their_packet = my_packet.clone();

    let main = move || {
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
  id: ThreadId,
}

#[derive(Clone)]
pub struct Thread {
  inner: Arc<Inner>,
}

impl Thread {
  pub fn new() -> Thread {
    Thread {
      inner: Arc::new(Inner {
        id: ThreadId::new(),
      })
    }
  }
  pub fn id(&self) -> ThreadId {
    self.inner.id
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
  pub fn native(&self) -> usize {
    self.0.native.as_ref().unwrap().id()
  }
  pub fn join(mut self) -> Result<T> {
    self.0.join()
  }
}
