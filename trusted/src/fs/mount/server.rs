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
          let fs = match packet.a {
            SYS_OPEN
            | SYS_CHMOD
            | SYS_RMDIR
            | SYS_UNLINK
            => {
              let s = ForeignSlice::new(asid, packet.b, packet.c).unwrap();
              packet.b = s.local_start;
              Some(s)
            }
            SYS_DUP
            | SYS_READ
            | SYS_WRITE
            | SYS_FMAP_OLD
            | SYS_FMAP
            | SYS_FPATH
            | SYS_FRENAME
            | SYS_FSTAT
            | SYS_FSTATVFS
            | SYS_FUTIMENS
            => {
              let s = ForeignSlice::new(asid, packet.c, packet.d).unwrap();
              packet.c = s.local_start;
              Some(s)
            }
            SYS_LSEEK
            | SYS_FCHMOD
            | SYS_FCHOWN
            | SYS_FCNTL
            | SYS_FEVENT
            | SYS_FUNMAP_OLD
            | SYS_FUNMAP
            | SYS_FSYNC
            | SYS_FTRUNCATE
            | SYS_CLOSE
            => None,
            _ => panic!("NOT translated packet"),
          };
          scheme.handle(&mut packet);
          drop(fs);
        }

        let mut msg = Message::default();
        msg.a = packet.a;
        trace!("handle packet err {:?}", Error::demux(msg.a));
        let _ = msg.send_to(tid);
      }
    }
    Err(e) => { error!("FileSystem::open {:?}", e); }
  }
}

