use microcall::message::Message;
use libtrusted::mm::{Entry, EntryLike};
use microcall::get_tid;

pub fn server() {
  info!("server started t{}",  get_tid());
  microcall::server_register(common::server::SERVER_MM).unwrap();
  loop {
    let (client_tid, msg) = Message::receive().unwrap();
    trace!("t{}: {:x?}", client_tid, msg);
    let asid = microcall::get_asid(client_tid).unwrap();
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
    msg.send_to(client_tid);
  }
}