use crate::fs::mount::scheme::FileScheme;
use crate::fs::{VirtioClient, FileSystem};
use crate::syscall::{Packet, Scheme};
use crate::microcall::get_tid;
use alloc::string::String;
use crate::itc::*;
use crate::root::server_set_busy;
use crate::root::Server::RedoxFs;

pub fn server() {
  println!("[FS] server started t{}", get_tid());
  let disk = VirtioClient::new();
  match FileSystem::open(disk, Some(0)) {
    Ok(filesystem) => {
      let scheme = FileScheme::new(String::from("virtio"), filesystem);
      loop {
        let mut packet = Packet::default();

        server_set_busy(RedoxFs, false);
        let msg = ItcMessage::receive();
        server_set_busy(RedoxFs, true);
        packet.a = msg.1.a;
        packet.b = msg.1.b;
        packet.c = msg.1.c;
        packet.d = msg.1.d;
        let client = msg.0;
        println!("[FS] {:x?}", msg);
        scheme.handle(&mut packet);

        let mut msg = ItcMessage::default();
        msg.a = packet.a;
        loop {
          let r = msg.send_to(client);
          if r == 0 {
            println!("[FS] done");
            break;
          }
        }

      }
    }
    Err(e) => { println!("[FS] FileSystem::open {}", e.errno); }
  }
}

