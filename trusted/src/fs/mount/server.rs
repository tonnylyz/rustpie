use crate::fs::mount::scheme::FileScheme;
use crate::fs::{VirtioClient, FileSystem};
use libtrusted::redoxcall::*;
use alloc::string::String;
use libtrusted::message::Message;
use microcall::get_tid;

pub fn server() {
  println!("[FS] server started t{}", get_tid());
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
        // println!("[FS] from t{}: {:x?}", tid, msg);
        scheme.handle(&mut packet);

        let mut msg = Message::default();
        msg.a = packet.a;
        // println!("[FS] err {:?}", Error::demux(msg.a));
        loop {
          let r = msg.send_to(tid);
          // println!("[FS] reply to t{}: {:x?}", tid, msg);
          if r.is_ok() {
            // println!("[FS] done");
            break;
          }
        }

      }
    }
    Err(e) => { println!("[FS] FileSystem::open {}", e.errno); }
  }
}

