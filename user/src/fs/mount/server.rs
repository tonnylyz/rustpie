use crate::fs::mount::scheme::FileScheme;
use crate::fs::{VirtioClient, DiskCache, FileSystem, IS_UMT};
use crate::syscall::{Error, Packet, Scheme, SYS_OPEN, O_RDONLY, thread_alloc, get_tid};
use alloc::string::String;
use core::sync::atomic::Ordering;
use crate::itc::*;
use crate::mem::valloc;
use crate::config::PAGE_SIZE;

pub fn server() {
  let disk = VirtioClient::new();
  match FileSystem::open(disk, Some(0)) {
    Ok(filesystem) => {
      let scheme = FileScheme::new(String::from("virtio"), filesystem);
      println!("file server start loop");
      let stack = valloc(16);
      thread_alloc(test as usize, stack as usize + PAGE_SIZE * 16, get_tid() as usize);

      loop {
        if IS_UMT.load(Ordering::SeqCst) > 0 {
          break;
        }
        let mut packet = Packet::default();

        let mut msg = ItcMessage::receive();
        packet.a = msg.1.a;
        packet.b = msg.1.b;
        packet.c = msg.1.c;
        packet.d = msg.1.d;
        let client = msg.0;
        println!("{:#x?}", msg);
        scheme.handle(&mut packet);

        let mut msg = ItcMessage::default();
        msg.a = packet.a;
        msg.send_to(client);
      }
    }
    Err(e) => { println!("FileSystem::open {}", e.errno); }
  }
}

fn test(server_tid: u16) {
  println!("fs client test start t{}", get_tid());
  let filename = "CACHEDIR.TAG";
  let mut msg = ItcMessage::default();
  msg.a = SYS_OPEN;
  msg.b = filename.as_ptr() as usize;
  msg.c = filename.len();
  msg.d = O_RDONLY;
  loop {
    if msg.send_to(server_tid) == 0 {
      break;
    }
  }

  let (tid, msg) = ItcMessage::receive();
  println!("file client ok {}:{}", tid, msg.a as isize);
}