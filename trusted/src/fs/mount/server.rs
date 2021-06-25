use crate::fs::mount::scheme::FileScheme;
use crate::fs::{VirtioClient, FileSystem};
use libtrusted::redox::*;
use alloc::string::String;
use libtrusted::message::Message;
use microcall::{get_tid, get_asid};

pub fn server() {
  info!("server started t{}", get_tid());
  microcall::server_register(common::server::SERVER_REDOX_FS).unwrap();
  let disk = VirtioClient::new();
  match FileSystem::open(disk, Some(0)) {
    Ok(filesystem) => {
      let scheme = FileScheme::new(String::from("virtio"), filesystem);
      loop {
        let mut packet = Packet::default();
        let (tid, msg) = Message::receive();
        packet.a = msg.a;
        packet.b = msg.b;
        packet.c = msg.c;
        packet.d = msg.d;
        packet.pid = get_asid(tid) as usize;
        trace!("from t{}: {:x?}", tid, msg);
        scheme.handle(&mut packet);

        let mut msg = Message::default();
        msg.a = packet.a;
        trace!("handle packet err {:?}", Error::demux(msg.a));
        loop {
          let r = msg.send_to(tid);
          if r.is_ok() {
            break;
          }
        }
      }
    }
    Err(e) => { error!("FileSystem::open {}", e.errno); }
  }
}

