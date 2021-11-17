use libtrusted::thread;
use libtrusted::wrapper::server_wrapper;
use microcall::{get_tid, thread_destroy};
use microcall::message::Message;

#[allow(dead_code)]
#[cfg(target_arch = "aarch64")]
pub fn current_cycle() -> usize {
  let r;
  unsafe {
    asm!("mrs {}, pmccntr_el0", out(reg) r);
  }
  r
}

#[allow(dead_code)]
#[cfg(target_arch = "riscv64")]
pub fn current_cycle() -> usize {
  let r;
  unsafe {
    asm!("rdcycle {}", out(reg) r);
  }
  r
}

#[allow(dead_code)]
fn test_thread_switch() {
  let mut join_handlers = vec![];
  join_handlers.push(thread::spawn(|| {
    info!("t1: {}", get_tid());
    loop {
      microcall::thread_yield();
    }
  }));

  join_handlers.push(thread::spawn(|| {
    info!("t2: {}", get_tid());
    let mut msg = Message::default();
    for _ in 0..1000 {
      let icntr = current_cycle();
      microcall::thread_yield();
      let icntr2 = current_cycle();
      println!("{}", icntr2 - icntr);
    }
  }));

  let _ = thread_destroy(0);
}

pub fn main() {
  let mut join_handlers = vec![];

  join_handlers.push(thread::spawn(|| {
    server_wrapper(crate::test::server);
  }));

  #[cfg(not(feature = "k210"))]
    join_handlers.push(thread::spawn(|| {
    server_wrapper(crate::blk::virtio_blk::server);
  }));

  #[cfg(feature = "k210")]
    join_handlers.push(thread::spawn(|| {
    server_wrapper(crate::blk::k210_sdcard::server);
  }));

  join_handlers.push(thread::spawn(|| {
    server_wrapper(crate::rtc::server);
  }));

  join_handlers.push(thread::spawn(|| {
    server_wrapper(crate::fs::server);
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
    match libtrusted::loader::spawn("shell") {
      Ok((_asid, tid)) => {
        microcall::thread_set_status(tid, common::thread::THREAD_STATUS_RUNNABLE).expect("root start shell failed");
      }
      Err(s) => { error!("{}", s); }
    }
  }));
  // thread_destroy(0);
  for handler in join_handlers {
    handler.join().expect("root join thread failed");
  }
}
