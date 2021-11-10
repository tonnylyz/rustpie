
use libtrusted::thread;
use libtrusted::wrapper::server_wrapper;


#[cfg(target_arch = "aarch64")]
pub fn current_cycle() -> usize {
  let r;
  unsafe {
    asm!("mrs {}, pmccntr_el0", out(reg) r);
  }
  r
}

#[cfg(target_arch = "riscv64")]
pub fn current_cycle() -> usize {
  let r;
  unsafe {
    asm!("rdcycle {}", out(reg) r);
  }
  r
}

pub fn main() {
  // let icntr = current_cycle();
  // microcall::null();
  // let icntr2 = current_cycle();
  // info!("syscall cycle {}", icntr2 - icntr);

  // microcall::get_asid(0);
  // microcall::get_asid(0);
  // info!("[[RECOVERY]]");
  // loop{}
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
      Ok((asid, tid)) => {
        microcall::thread_set_status(tid, common::thread::THREAD_STATUS_RUNNABLE);
      }
      Err(s) => { error!("{}", s); }
    }
  }));

  for handler in join_handlers {
    handler.join();
  }
}
