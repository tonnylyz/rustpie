use fs::{File, SeekFrom};
use libtrusted::thread;
use libtrusted::wrapper::server_wrapper;
use unwind::catch::catch_unwind;

pub fn main() {
  thread::spawn(|| {
    server_wrapper(crate::test::server);
  });

  thread::spawn(|| {
    server_wrapper(crate::blk::virtio_blk::server);
  });

  thread::spawn(|| {
    server_wrapper(crate::fs::server);
  });

  thread::spawn(|| {
    server_wrapper(crate::terminal::input_server);
  });

  thread::spawn(|| {
    server_wrapper(crate::terminal::server);
  });

  thread::spawn(|| {
    server_wrapper(crate::mm::server);
  });

  thread::spawn(|| {
    server_wrapper(crate::pm::server);
  });

  thread::spawn(|| {
    server_wrapper(crate::pm::event_server);
  });

  thread::spawn(|| {
    match libtrusted::loader::spawn("shell") {
      Ok(_) => {}
      Err(s) => { error!("{}", s); }
    }
    microcall::thread_destroy(0);
  });

  loop {
    microcall::thread_yield();
  }
}
