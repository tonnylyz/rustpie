use crate::fs::mount::scheme::FileScheme;
use crate::fs::{VirtioClient, FileSystem};
use redox::*;
use alloc::string::String;
use microcall::message::Message;
use microcall::{get_tid, get_asid};
use libtrusted::foreign_slice::ForeignSlice;

pub fn server() {
  info!("server started t{}", get_tid());
  microcall::server_register(common::server::SERVER_REDOX_FS).unwrap();
  let disk = VirtioClient::new();
  match FileSystem::open(disk, Some(0)) {
    Ok(filesystem) => {
      let scheme = FileScheme::new(String::from("virtio"), filesystem);
      loop {
        let mut packet = Packet::default();
        let (tid, msg) = Message::receive().unwrap();
        packet.a = msg.a;
        packet.b = msg.b;
        packet.c = msg.c;
        packet.d = msg.d;
        let asid = get_asid(tid).unwrap();
        trace!("from t{}: {:x?}", tid, msg);
        if asid == get_asid(0).unwrap() {
          scheme.handle(&mut packet);
        } else {
          match packet.a {
            SYS_OPEN | SYS_UNLINK => {
              let s = ForeignSlice::new(asid, packet.b, packet.c).unwrap();
              packet.b = s.local_start;
            }
            SYS_READ | SYS_WRITE => {
              let s = ForeignSlice::new(asid, packet.c, packet.d).unwrap();
              packet.c = s.local_start;
            }
            SYS_CLOSE => {}
            _ => panic!("NOT translated packet"),
          }
          scheme.handle(&mut packet);
        }

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

