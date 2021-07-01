use alloc::boxed::Box;
use core::mem::ManuallyDrop;

pub type PanicError = &'static str;

pub fn catch_unwind<F: FnOnce() -> R, R>(f: F) -> Result<R, PanicError> {
  unsafe { r#try(f) }
}

/// Invoke a closure, capturing the cause of an unwinding panic if one occurs.
pub unsafe fn r#try<R, F: FnOnce() -> R>(f: F) -> Result<R, PanicError> {
  struct Data<F, R> {
    f: ManuallyDrop<F>,
    r: ManuallyDrop<Result<R, PanicError>>,
  }

  let mut data = Data {
    f: ManuallyDrop::new(f),
    r: ManuallyDrop::new(Err("Catch failed"))
  };

  let data_ptr = &mut data as *mut _ as *mut u8;
  let _r = core::intrinsics::r#try(
    do_call::<F, R>,
    data_ptr,
    do_catch::<F, R>
  );
  return ManuallyDrop::into_inner(data.r);


  #[inline]
  fn do_call<F: FnOnce() -> R, R>(data: *mut u8) {
    unsafe {
      let data = data as *mut Data<F, R>;
      let data = &mut (*data);
      let f = ManuallyDrop::take(&mut data.f);
      data.r = ManuallyDrop::new(Ok(f()));
    }
  }

  #[inline]
  fn do_catch<F: FnOnce() -> R, R>(data: *mut u8, payload: *mut u8) {
    unsafe {
      let data = data as *mut Data<F, R>;
      let data = &mut *data;
      let unwinding_context_boxed = Box::from_raw(payload as *mut super::UnwindingContext);
      let _unwinding_context = *unwinding_context_boxed;
      data.r = ManuallyDrop::new(Err("unwinding_context"));
    }
  }
}
