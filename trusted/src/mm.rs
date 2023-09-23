use crate::common::mm::default_page_attribute;
use crate::common::wrapper::request_wrapper;
use rpsyscall::get_tid;
use rpsyscall::message::Message;

fn mm(msg: Message, tid: usize) -> usize {
  let asid = rpsyscall::get_asid(tid).unwrap();
  match msg.a {
    rpservapi::mm::action::ALLOC => {
      match rpsyscall::mem_alloc(asid, msg.b, default_page_attribute()) {
        Ok(_) => rpservapi::mm::result::OK,
        Err(_) => rpservapi::mm::result::ERR
      }
    }
    _ => {
      rpservapi::mm::result::UNKNOWN_ACTION
    }
  }
}

pub fn server() {
  info!("server started t{}",  get_tid());
  rpsyscall::server_register(rpabi::server::SERVER_MM).unwrap();
  loop {
    let (client_tid, msg) = Message::receive().unwrap();
    let r = request_wrapper(mm, msg, client_tid).unwrap();
    let result = Message::new(r, 0, 0, 0);
    let _ = result.send_to(client_tid);
  }
}
