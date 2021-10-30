
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
