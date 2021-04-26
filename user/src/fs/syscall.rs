use crate::itc::*;
use crate::syscall::{SYS_OPEN, O_RDONLY, SYS_READ, SYS_LSEEK};
use spin::Once;

pub static FS_SERVER_TID: Once<u16> = Once::new();

fn server_tid() -> u16 {
  loop {
    if let Some(tid) = FS_SERVER_TID.get() {
      break *tid;
    }
  }
}

pub enum Error {
  NONE
}

pub type Result<T> = core::result::Result<T, Error>;

pub struct File {
  handle: usize,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SeekFrom {
  Start(u64),
  End(i64),
  Current(i64),
}

pub const SEEK_SET: usize = 0;
pub const SEEK_END: usize = 1;
pub const SEEK_CUR: usize = 2;

impl File {

  pub fn open<P: AsRef<str>>(path: P) -> Result<File> {
    let mut msg = ItcMessage::default();
    msg.a = SYS_OPEN;
    msg.b = path.as_ref().as_ptr() as usize;
    msg.c = path.as_ref().len();
    msg.d = O_RDONLY;
    loop {
      if msg.send_to(server_tid()) == 0 {
        break;
      }
    }
    let (_tid, msg) = ItcMessage::receive();
    let err = crate::syscall::Error::demux(msg.a);
    match err {
      Ok(handle) => { Ok(File{handle}) }
      Err(_) => { Err(Error::NONE) }
    }
  }

  pub fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
    let mut msg = ItcMessage::default();
    msg.a = SYS_READ;
    msg.b = self.handle;
    msg.c = buf.as_ptr() as usize;
    msg.d = buf.len();
    loop {
      if msg.send_to(server_tid()) == 0 {
        break;
      }
    }
    let (_tid, msg) = ItcMessage::receive();
    let err = crate::syscall::Error::demux(msg.a);
    match err {
      Ok(read) => { Ok(read) }
      Err(_) => { Err(Error::NONE) }
    }
  }

  pub fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
    let mut msg = ItcMessage::default();
    msg.a = SYS_LSEEK;
    msg.b = self.handle;
    msg.c =
      match pos {
        SeekFrom::Start(u) => {u as usize}
        SeekFrom::End(i) => {i as usize}
        SeekFrom::Current(i) => {i as usize}
      };
    msg.d =
      match pos {
        SeekFrom::Start(u) => {SEEK_SET}
        SeekFrom::End(i) => {SEEK_END}
        SeekFrom::Current(i) => {SEEK_CUR}
      };
    loop {
      if msg.send_to(server_tid()) == 0 {
        break;
      }
    }
    let (_tid, msg) = ItcMessage::receive();
    let err = crate::syscall::Error::demux(msg.a);
    match err {
      Ok(p) => { Ok(p as u64) }
      Err(_) => { Err(Error::NONE) }
    }
  }
}