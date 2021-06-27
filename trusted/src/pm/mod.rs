use microcall::message::Message;
use libtrusted::mm::{Entry, EntryLike};
use microcall::{get_tid, get_asid, mem_map, mem_unmap};
use common::PAGE_SIZE;
use alloc::string::String;
use libtrusted::foreign_slice::ForeignSlice;

fn process_request(asid: u16, msg: &Message) -> Result<(), &'static str> {
  match msg.a {
    1 => { // SPAWN
      let length = msg.c;
      if length == 0 {
        return Ok(())
      }
      if length >= 128 {
        return Err("MalformedString");
      }
      let s = ForeignSlice::new(asid, msg.b, msg.c).unwrap();
      let cmd = s.local_slice();
      let cmd = core::str::from_utf8(cmd).map_err(|_| "MalEncoded")?;
      let asid = libtrusted::loader::spawn(cmd)?;
      Ok(())
    }
    _ => {
      Err("UnknownRequest")
    }
  }
}

pub fn server() {
  info!("server started t{}", get_tid());
  microcall::server_register(common::server::SERVER_PM).unwrap();
  loop {
    let (tid, msg) = Message::receive().unwrap();
    trace!("t{}: {:x?}", tid, msg);
    let asid = get_asid(tid);
    match process_request(asid, &msg) {
      Ok(_) => {
        let mut msg = Message::default();
        msg.a = 0;
        msg.send_to(tid).unwrap();
      }
      Err(e) => {
        error!("ERR {}", e);
        let mut msg = Message::default();
        msg.a = 1;
        msg.send_to(tid).unwrap();
      }
    }
  }
}