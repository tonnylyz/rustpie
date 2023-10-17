use rpsyscall::get_tid;

pub fn server() {
  info!("server started t{}", get_tid());
  rpsyscall::server_register(rpabi::server::SERVER_RTC).unwrap();
  loop {
    let (client_tid, _msg) = rpsyscall::message::Message::receive().unwrap();
    let msg = rpsyscall::message::Message::default();
    let _ = msg.send_to(client_tid);
  }
}
