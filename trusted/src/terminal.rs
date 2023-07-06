use alloc::collections::VecDeque;

use spin::{Mutex, Once};

use rpsyscall::get_tid;
use rpsyscall::message::Message;

pub fn input_server() {
  loop {
    if let Ok(c) = rpsyscall::getc() {
      let mut buf = buffer().lock();
      buf.push_back(c);
    }
    rpsyscall::thread_yield();
  }
}

static BUFFER: Once<Mutex<VecDeque<u8>>> = Once::new();

fn buffer() -> &'static Mutex<VecDeque<u8>> {
  match BUFFER.get() {
    None => { BUFFER.call_once(|| Mutex::new(VecDeque::new())) }
    Some(x) => { x }
  }
}

pub fn server() {
  info!("server started t{}",  get_tid());
  rpsyscall::server_register(rpabi::server::SERVER_TERMINAL).unwrap();
  let mut client_tid;
  client_tid = Message::receive().unwrap().0;
  loop {
    let mut msg = rpsyscall::message::Message::default();
    let mut buf = buffer().lock();
    match buf.pop_front() {
      None => { msg.a = 0 }
      Some(c) => { msg.a = c as usize }
    }
    drop(buf);
    client_tid = msg.reply_recv(client_tid).unwrap().0;
  }
}