use libtrusted::message::Message;
use libtrusted::mm::{Entry, EntryLike};
use microcall::get_tid;

pub fn server() {
  println!("[MM] server started t{}",  get_tid());
  microcall::server_register(common::server::SERVER_MM).unwrap();
  loop {
    let (tid, msg) = Message::receive();
    // println!("[MM] t{}: {:x?}", tid, msg);
    let asid = microcall::get_asid(tid);
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
    msg.reply()
  }
}