// root task

use spin::Mutex;
use alloc::collections::BTreeMap;
use crate::thread;
use crate::thread::JoinHandle;
use crate::microcall::{get_tid, thread_destroy};
use crate::fs::{File, SeekFrom};

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum Server {
  VirtioBlk = 0,
  RedoxFs = 1,
}

static HANDLES: Mutex<BTreeMap<Server, JoinHandle<()>>> = Mutex::new(BTreeMap::new());

static BUSY: Mutex<BTreeMap<Server, bool>> = Mutex::new(BTreeMap::new());

pub fn main(_arg: usize) {
  let mut handles = HANDLES.lock();
  handles.insert(Server::VirtioBlk, thread::spawn(|| {
    crate::virtio_blk::server();
  }));

  handles.insert(Server::RedoxFs, thread::spawn(|| {
    crate::fs::server();
  }));

  drop(handles);

  thread::spawn(|| {
    println!("[TEST] client t{}", get_tid());
    let mut file = File::open("hello").ok().unwrap();
    let mut buf: [u8; 128] = [0; 128];
    let r = file.seek(SeekFrom::Start(0)).ok().unwrap();
    println!("[TEST] client seek {}", r);
    let r = file.read(&mut buf).ok().unwrap();
    println!("[TEST] client read {}", r);
    let str = core::str::from_utf8(&buf).unwrap();
    println!("[TEST] client str {}", str);
    thread_destroy(0);
  });

  loop {}
}

pub fn server_tid(server: Server) -> Result<u16, ()> {
  let handles = HANDLES.lock();
  if let Some(handle) = handles.get(&server) {
    Ok(handle.native())
  } else {
    Err(())
  }
}

pub fn server_tid_wait(server: Server) -> u16 {
  loop {
    if let Ok(tid) = server_tid(server) {
      break tid
    }
  }
}

pub fn server_set_busy(server: Server, b: bool) {
  let mut busy = BUSY.lock();
  busy.insert(server, b);
}

pub fn server_busy(server: Server) -> bool {
  let busy = BUSY.lock();
  *busy.get(&server).unwrap_or_else(|| &true)
}
