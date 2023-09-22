use alloc::string::String;

use rpabi::PAGE_SIZE;
use spin::Once;

use crate::libtrusted::foreign_slice::ForeignSlice;
use crate::libtrusted::mm::virtual_alloc;
use crate::libtrusted::wrapper::request_wrapper;
use rpsyscall::{get_asid, get_tid};
use rpsyscall::message::Message;
use syscall::{Packet, Scheme};

use crate::fs::disk::VirtioClient;
use redoxfs::FileScheme;

#[no_mangle]
pub extern "C" fn memalign(_align: usize, size: usize) -> *mut u8 {
  virtual_alloc(size / PAGE_SIZE, true).unwrap() as *mut u8
}

#[no_mangle]
pub extern "C" fn free(_ptr: *mut u8) {
  // TODO: implement free
}

fn redoxfs_proc(msg: Message, client_tid: usize) -> usize {
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
      syscall::SYS_OPEN
      | syscall::SYS_CHMOD
      | syscall::SYS_RMDIR
      | syscall::SYS_UNLINK
      => {
        let s = ForeignSlice::new(asid, packet.b, packet.c).unwrap();
        packet.b = s.local_start;
        Some(s)
      }
      syscall::SYS_DUP
      | syscall::SYS_READ
      | syscall::SYS_WRITE
      | syscall::SYS_FMAP_OLD
      | syscall::SYS_FMAP
      | syscall::SYS_FPATH
      | syscall::SYS_FRENAME
      | syscall::SYS_FSTAT
      | syscall::SYS_FSTATVFS
      | syscall::SYS_FUTIMENS
      => {
        let s = ForeignSlice::new(asid, packet.c, packet.d).unwrap();
        packet.c = s.local_start;
        Some(s)
      }
      syscall::SYS_LSEEK
      | syscall::SYS_FCHMOD
      | syscall::SYS_FCHOWN
      | syscall::SYS_FCNTL
      | syscall::SYS_FEVENT
      | syscall::SYS_FUNMAP_OLD
      | syscall::SYS_FUNMAP
      | syscall::SYS_FSYNC
      | syscall::SYS_FTRUNCATE
      | syscall::SYS_CLOSE
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
  match redoxfs::FileSystem::open(disk, Some(0)) {
    Ok(filesystem) => {
      FILE_SCHEME.call_once(|| { FileScheme::new(String::from("virtio"), filesystem) });
      loop {
        let (client_tid, msg) = Message::receive().unwrap();
        let r = request_wrapper(redoxfs_proc, msg, client_tid).unwrap();
        let result = Message::new(r, 0, 0, 0);
        let _ = result.send_to(client_tid);
      }
    }
    Err(e) => { error!("FileSystem::open {:?}", e); }
  }
}
