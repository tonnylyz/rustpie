use microcall::message::Message;
use libtrusted::mm::{Entry, EntryLike};
use microcall::get_tid;
use libtrusted::wrapper::{request_wrapper, Error};

// static mut PANICKED: bool = false;

fn process(msg: Message, tid: usize) -> () {
  trace!("t{}: {:x?}", tid, msg);
  let asid = microcall::get_asid(tid).unwrap();
  let r = match msg.a {
    1 => {
      microcall::mem_alloc(asid, msg.b, Entry::default().attribute());
      0
    },
    _ => {
      0x1
    }
  };

  let mut msg = Message::default();
  msg.a = r;
  msg.send_to(tid);
}

pub fn server() {
  info!("server started t{}",  get_tid());
  microcall::server_register(common::server::SERVER_MM).unwrap();
  loop {
    let (client_tid, msg) = Message::receive().unwrap();
    request_wrapper(process, msg, client_tid).unwrap();
  }
}