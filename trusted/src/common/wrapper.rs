use rpsyscall::message::Message;

#[cfg(feature = "error_unwind")]
use unwind::catch::catch_unwind;

pub type Error = usize;

#[cfg(feature = "error_unwind")]
pub const ERROR_PERSISTENT_FAILURE: usize = 0x999;


#[cfg(feature = "error_unwind")]
const RETRY_MAX: usize = 0;

#[cfg(feature = "error_unwind")]
pub fn request_wrapper<F: Fn(Message, usize) -> R, R>(f: F, msg: Message, tid: usize) -> Result<R, Error> {
  let mut i = 0;
  loop {
    i += 1;
    let r = catch_unwind(|| {
      f(msg, tid)
    });
    if r.is_ok() {
      break Ok(r.unwrap());
    } else {
      // info!("retry #{}", i);
      if i > RETRY_MAX {
        break Err(ERROR_PERSISTENT_FAILURE);
      }
    }
  }
}

#[cfg(feature = "error_unwind")]
pub fn server_wrapper<F: Fn()>(f: F) {
  loop {
    let r = catch_unwind(|| {
      f();
    });
    match r {
      Ok(_) => {
        error!("server should not return");
      }
      Err(_) => {
        error!("server died");
      }
    }
  }
}

#[cfg(not(feature = "error_unwind"))]
pub fn request_wrapper<F: Fn(Message, usize) -> R, R>(f: F, msg: Message, tid: usize) -> Result<R, Error> {
  Ok(f(msg, tid))
}

#[cfg(not(feature = "error_unwind"))]
pub fn server_wrapper<F: Fn()>(f: F) {
  f()
}