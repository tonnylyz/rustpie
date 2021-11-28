use alloc::string::String;
use libtrusted::{rendezvous, thread};
use libtrusted::wrapper::server_wrapper;
use microcall::{get_tid, thread_destroy};
use microcall::message::Message;
use unwind::catch::catch_unwind;

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


fn ipc_send_task(sender: rendezvous::Sender<String>) -> Result<(), &'static str> {
  info!("send task started t{}", get_tid());
  let mut i = 0;
  loop {
    let msg = format!("{:08}", i);
    sender.send(msg)?;
    i = i + 1;
  }
  Ok(())
}

fn ipc_receive_task((receiver, test): (rendezvous::Receiver<String>, usize)) -> Result<(), &'static str> {
  info!("receive task started t{}", get_tid());
  loop {
    let msg_final = format!("{:08}", 10);
    let msg = receiver.receive()?;
    if msg_final == msg {
      info!("TEST RESULT {} receive task successfully completed", test);
      loop {}
    }
  }
  Ok(())
}

fn spawn_ipc_fault_test() {
  // S1=> 1, Random page fault induced in sender routine
  // S2=> 12, Message pointer sent to null in sending routine
  // S3=> 2, Message pointer set to unmapped address in sending routine
  // S4=> 3, Sender pointer set to unmapped process in send routine
  // S5=> 5, Wait queue set to unmapped address in send routine
  // S6=> 11, Empty channel not in initial state when send begin TODO! (expected)
  // S7=> 13, State not updated after transmitting message by sender TODO!
  // R1=> 4, Receive pointer set to unmapped process in receive routine
  // R2=> 6, Empty channel not in initial state when receive begin
  // R3=> 7, Random page fault induced in receiver routine
  // R4=> 8, Receive msg buffer set to null
  // R5=> 9, Wait queue set to unmapped address in receiver routine
  // R6=> 10, An empty slot marked as occupied

  let fault_num = 10;
  rendezvous::set_fault_item(fault_num);
  // create two channels to pass messages
  let (sender, receiver) = rendezvous::new_channel::<String>();
  let mut join_handlers = vec![];

  join_handlers.push(thread::spawn(move || {
    loop {
      if let Ok(Ok(_)) = catch_unwind(|| {
        ipc_send_task(sender.clone())
      }) {
        break
      }
    }
  }));

  join_handlers.push(thread::spawn(move || {
    loop {
      if let Ok(Ok(_)) = catch_unwind(|| {
        ipc_receive_task((receiver.clone(), fault_num))
      }) {
        break
      }
    }
  }));

  for handler in join_handlers {
    handler.join().expect("root join thread failed");
  }
}


pub fn main() {
  let mut join_handlers = vec![];
  // spawn_ipc_fault_test();

  join_handlers.push(thread::spawn(|| {
    server_wrapper(crate::test::server);
  }));

  join_handlers.push(thread::spawn(|| {
    server_wrapper(crate::blk::server);
  }));

  #[cfg(not(feature = "tx2"))]
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
