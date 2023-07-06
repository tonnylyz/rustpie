use alloc::string::String;

use spin::Once;

use crate::libtrusted::foreign_slice::ForeignSlice;
use crate::libtrusted::wrapper::request_wrapper;
use rpsyscall::{get_asid, get_tid};
use rpsyscall::message::Message;
use redox::*;

use crate::fs::{FileSystem, VirtioClient};
use crate::fs::mount::scheme::FileScheme;

fn redoxfs(msg: Message, client_tid: usize) -> usize {
  let mut packet = Packet::default();
  packet.a = msg.a;
  packet.b = msg.b;
  packet.c = msg.c;
  packet.d = msg.d;
  let asid = get_asid(client_tid).unwrap();
  if asid == get_asid(0).unwrap() {
    FILE_SCHEME.get().unwrap().handle(&mut packet);
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
    FILE_SCHEME.get().unwrap().handle(&mut packet);
    drop(fs);
  }
  packet.a
}

static FILE_SCHEME: Once<FileScheme<VirtioClient>> = Once::new();

pub fn server() {
  info!("server started t{}", get_tid());
  rpsyscall::server_register(rpabi::server::SERVER_REDOX_FS).unwrap();
  let disk = VirtioClient::new();
  match FileSystem::open(disk, Some(0)) {
    Ok(filesystem) => {
      FILE_SCHEME.call_once(|| { FileScheme::new(String::from("virtio"), filesystem) });
      loop {
        let (client_tid, msg) = Message::receive().unwrap();
        let r = request_wrapper(redoxfs, msg, client_tid).unwrap();
        let result = Message::new(r, 0, 0, 0);
        let _ = result.send_to(client_tid);
      }
    }
    Err(e) => { error!("FileSystem::open {:?}", e); }
  }
}
