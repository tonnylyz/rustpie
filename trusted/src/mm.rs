use microcall::message::Message;
use libtrusted::mm::default_page_attribute;
use microcall::get_tid;
use libtrusted::wrapper::request_wrapper;

// static mut PANICKED: bool = false;

fn process(msg: Message, tid: usize) -> () {
  trace!("t{}: {:x?}", tid, msg);
  let asid = microcall::get_asid(tid).unwrap();
  let r = match msg.a {
    1 => {
      match microcall::mem_alloc(asid, msg.b, default_page_attribute()) {
        Ok(_) => {0}
        Err(_) => {0x1}
      }
    },
    _ => {
      0x1
    }
  };

  let mut msg = Message::default();
  msg.a = r;
  let _ = msg.send_to(tid);
}

pub fn server() {
  info!("server started t{}",  get_tid());
  microcall::server_register(common::server::SERVER_MM).unwrap();
  loop {
    let (client_tid, msg) = Message::receive().unwrap();
    request_wrapper(process, msg, client_tid).unwrap();
  }
}