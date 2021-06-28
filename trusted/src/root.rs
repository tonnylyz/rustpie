use fs::{File, SeekFrom};
use libtrusted::thread;

pub fn main() {
  thread::spawn(|| {
    crate::blk::virtio_blk::server();
  });

  thread::spawn(|| {
    crate::fs::server();
  });

  thread::spawn(|| {
    crate::terminal::input_server();
  });

  thread::spawn(|| {
    crate::terminal::server();
  });

  thread::spawn(|| {
    crate::mm::server();
  });

  thread::spawn(|| {
    crate::pm::server();
  });

  thread::spawn(|| {
    crate::pm::event_server();
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
