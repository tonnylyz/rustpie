use crate::fs::mount::scheme::FileScheme;
use crate::fs::{VirtioClient, DiskCache, FileSystem, IS_UMT, FS_SERVER_TID, File, SeekFrom};
use crate::syscall::{Error, Packet, Scheme, SYS_OPEN, O_RDONLY, thread_alloc, get_tid, thread_destroy};
use alloc::string::String;
use core::sync::atomic::Ordering;
use crate::itc::*;
use crate::mem::valloc;
use crate::config::PAGE_SIZE;

pub fn server() {
  println!("[FS] server started t{}", get_tid());
  let disk = VirtioClient::new();
  match FileSystem::open(disk, Some(0)) {
    Ok(filesystem) => {
      let scheme = FileScheme::new(String::from("virtio"), filesystem);
      FS_SERVER_TID.call_once(|| get_tid());
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
        // println!("{:#x?}", msg);
        scheme.handle(&mut packet);

        let mut msg = ItcMessage::default();
        msg.a = packet.a;
        msg.send_to(client);
      }
    }
    Err(e) => { println!("[FS] FileSystem::open {}", e.errno); }
  }
}

fn test() {
  let mut file = File::open("hello").ok().unwrap();
  let mut buf: [u8; 128] = [0; 128];
  let r = file.seek(SeekFrom::Start(0)).ok().unwrap();
  println!("r {}", r);
  let r = file.read(&mut buf).ok().unwrap();
  println!("r {}", r);
  for i in 0..128 {
    print!("{:02x }", buf[i]);
  }
  let str = core::str::from_utf8(&buf).unwrap();
  println!("FS TEST {}", str);
  thread_destroy(0);
}