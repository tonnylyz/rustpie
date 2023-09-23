use crate::common::thread;
use crate::common::wrapper::server_wrapper;

pub fn main() {
  let mut join_handlers = vec![];

  join_handlers.push(thread::spawn(|| {
    server_wrapper(crate::blk::server);
  }));

  #[cfg(not(feature = "tx2"))]
  join_handlers.push(thread::spawn(|| {
    server_wrapper(crate::rtc::server);
  }));

  join_handlers.push(thread::spawn(|| {
    server_wrapper(crate::fs::server::server);
  }));

  join_handlers.push(thread::spawn(|| {
    server_wrapper(crate::terminal::input_server);
  }));

  join_handlers.push(thread::spawn(|| {
    server_wrapper(crate::terminal::server);
  }));

  join_handlers.push(thread::spawn(|| {
    server_wrapper(crate::mm::server);
  }));

  join_handlers.push(thread::spawn(|| {
    server_wrapper(crate::pm::server);
  }));

  join_handlers.push(thread::spawn(|| {
    match crate::common::loader::spawn("shell") {
      Ok((_asid, tid)) => {
        rpsyscall::thread_set_status(tid, rpabi::thread::THREAD_STATUS_RUNNABLE).expect("root start shell failed");
      }
      Err(s) => { error!("{}", s); }
    }
  }));
  for handler in join_handlers {
    handler.join().expect("root join thread failed");
  }
}
