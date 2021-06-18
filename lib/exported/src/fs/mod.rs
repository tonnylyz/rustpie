use trusted::message::Message;
use trusted::redoxcall::*;

pub enum Error {
  NONE
}

pub type Result<T> = core::result::Result<T, Error>;

pub struct File {
  handle: usize,
}

#[allow(dead_code)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SeekFrom {
  Start(u64),
  End(i64),
  Current(i64),
}

impl File {

  pub fn open<P: AsRef<str>>(path: P) -> Result<File> {
    let mut msg = Message::default();
    msg.a = SYS_OPEN;
    msg.b = path.as_ref().as_ptr() as usize;
    msg.c = path.as_ref().len();
    msg.d = O_RDONLY;
    let msg = msg.call(microcall::server_tid_wait(common::server::SERVER_REDOX_FS));
    let err = trusted::redoxcall::Error::demux(msg.a);
    match err {
      Ok(handle) => { Ok(File{handle}) }
      Err(_) => { Err(Error::NONE) }
    }
  }

  pub fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
    let mut msg = Message::default();
    msg.a = SYS_READ;
    msg.b = self.handle;
    msg.c = buf.as_ptr() as usize;
    msg.d = buf.len();
    let msg = msg.call(microcall::server_tid_wait(common::server::SERVER_REDOX_FS));
    let err = trusted::redoxcall::Error::demux(msg.a);
    match err {
      Ok(read) => { Ok(read) }
      Err(_) => { Err(Error::NONE) }
    }
  }

  pub fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
    let mut msg = Message::default();
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
        SeekFrom::Start(_u) => {SEEK_SET}
        SeekFrom::End(_i) => {SEEK_END}
        SeekFrom::Current(_i) => {SEEK_CUR}
      };
    let msg = msg.call(microcall::server_tid_wait(common::server::SERVER_REDOX_FS));
    let err = trusted::redoxcall::Error::demux(msg.a);
    match err {
      Ok(p) => { Ok(p as u64) }
      Err(_) => { Err(Error::NONE) }
    }
  }
}