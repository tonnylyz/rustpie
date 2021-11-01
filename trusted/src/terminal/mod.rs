
use spin::{Mutex, Once};
use alloc::collections::VecDeque;
use microcall::get_tid;

// #[cfg(target_arch = "aarch64")]
// mod pl011;
//
// #[cfg(target_arch = "aarch64")]
// pub fn input_server() {
//   pl011::enable_irq();
//   loop {
//     microcall::event_wait(common::event::EVENT_INTERRUPT, 0x1 + 32);
//     pl011::irq();
//   }
// }

// #[cfg(target_arch = "riscv64")]
pub fn input_server() {
  loop {
    if let Ok(c) = microcall::getc() {
      let mut buf = buffer().lock();
      buf.push_back(c);
    }
    microcall::thread_yield();
  }
}

static BUFFER: Once<Mutex<VecDeque<u8>>> = Once::new();

fn buffer() -> &'static Mutex<VecDeque<u8>> {
  match BUFFER.get() {
    None => { BUFFER.call_once(|| Mutex::new(VecDeque::new())); buffer() }
    Some(x) => {x}
  }
}

pub fn server() {
  info!("server started t{}",  get_tid());
  microcall::server_register(common::server::SERVER_TERMINAL).unwrap();

  loop {
    let (client_tid, _msg) = microcall::message::Message::receive().unwrap();
    let mut msg = microcall::message::Message::default();
    let mut buf = buffer().lock();
    match buf.pop_front() {
      None => { msg.a = 0 }
      Some(c) => { msg.a = c as usize }
    }
    let _ = msg.send_to(client_tid);
  }
}