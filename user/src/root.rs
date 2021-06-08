use alloc::collections::BTreeMap;
use spin::Mutex;
use trusted::thread::{JoinHandle, spawn};

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum Server {
  VirtioBlk = 0,
  RedoxFs = 1,
}

static HANDLES: Mutex<BTreeMap<Server, JoinHandle<()>>> = Mutex::new(BTreeMap::new());

pub fn main(_arg: usize) {
  let mut handles = HANDLES.lock();
  handles.insert(Server::VirtioBlk, spawn(|| {
    crate::blk::virtio_blk::server();
  }));

  handles.insert(Server::RedoxFs, spawn(|| {
    crate::fs::server();
  }));

  drop(handles);
  loop {
    microcall::thread_yield();
  }
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
