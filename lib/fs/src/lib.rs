#![no_std]

use microcall::message::Message;
use redox::*;
use common::server::SERVER_REDOX_FS;

pub use redox::Stat;

pub struct File {
  handle: usize,
}

pub struct Permissions(u16);

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SeekFrom {
  Start(u64),
  End(i64),
  Current(i64),
}

impl File {
  pub fn open<P: AsRef<str>>(path: P) -> Result<File> {
    let msg = Message {
      a: SYS_OPEN,
      b: path.as_ref().as_ptr() as usize,
      c: path.as_ref().len(),
      d: O_RDONLY,
    };
    let msg = msg.call(SERVER_REDOX_FS).map_err(|_| Error::new(EIO))?;
    Error::demux(msg.a).map(|handle| File{ handle })
  }

  pub fn create<P: AsRef<str>>(path: P) -> Result<File> {
    let msg = Message {
      a: SYS_OPEN,
      b: path.as_ref().as_ptr() as usize,
      c: path.as_ref().len(),
      d: O_CREAT | O_RDWR,
    };
    let msg = msg.call(SERVER_REDOX_FS).map_err(|_| Error::new(EIO))?;
    Error::demux(msg.a).map(|handle| File{ handle })
  }

  pub fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
    let msg = Message {
      a: SYS_READ,
      b: self.handle,
      c: buf.as_ptr() as usize,
      d: buf.len(),
    };
    let msg = msg.call(SERVER_REDOX_FS).map_err(|_| Error::new(EIO))?;
    Error::demux(msg.a)
  }

  pub fn write(&mut self, buf: &[u8]) -> Result<usize> {
    let msg = Message {
      a: SYS_WRITE,
      b: self.handle,
      c: buf.as_ptr() as usize,
      d: buf.len(),
    };
    let msg = msg.call(SERVER_REDOX_FS).map_err(|_| Error::new(EIO))?;
    Error::demux(msg.a)
  }

  pub fn seek(&self, pos: SeekFrom) -> Result<u64> {
    let msg = Message {
      a: SYS_LSEEK,
      b: self.handle,
      c: match pos {
        SeekFrom::Start(u) => u as usize,
        SeekFrom::End(i) => i as usize,
        SeekFrom::Current(i) => i as usize,
      },
      d: match pos {
        SeekFrom::Start(_u) => SEEK_SET,
        SeekFrom::End(_i) => SEEK_END,
        SeekFrom::Current(_i) => SEEK_CUR,
      }
    };
    let msg = msg.call(SERVER_REDOX_FS).map_err(|_| Error::new(EIO))?;
    Error::demux(msg.a).map(|u| u as u64)
  }

  pub fn set_len(&self, size: u64) -> Result<()> {
    let msg = Message {
      a: SYS_FTRUNCATE,
      b: self.handle,
      c: size as usize,
      d: 0,
    };
    let msg = msg.call(SERVER_REDOX_FS).map_err(|_| Error::new(EIO))?;
    Error::demux(msg.a).map(|_| ())
  }

  pub fn stat(&self) -> Result<Stat> {
    let mut stat = Stat::default();
    let msg = Message {
      a: SYS_FSTAT,
      b: self.handle,
      c: (&mut stat).as_mut_ptr() as usize,
      d: (&stat).len(),
    };
    let msg = msg.call(SERVER_REDOX_FS).map_err(|_| Error::new(EIO))?;
    Error::demux(msg.a).map(|_| stat)
  }

  pub fn set_permissions(&self, perm: Permissions) -> Result<()> {
    let msg = Message {
      a: SYS_CHMOD,
      b: self.handle,
      c: perm.0 as usize,
      d: 0,
    };
    let msg = msg.call(SERVER_REDOX_FS).map_err(|_| Error::new(EIO))?;
    Error::demux(msg.a).map(|_| ())
  }

}

impl Drop for File {
  fn drop(&mut self) {
    let msg = Message {
      a: SYS_CLOSE,
      b: self.handle,
      c: 0,
      d: 0,
    };
    let _ = msg.call(SERVER_REDOX_FS).map_err(|_| Error::new(EIO));
  }
}

pub fn create_dir<P: AsRef<str>>(path: P) -> Result<()> {
  let msg = Message {
    a: SYS_OPEN,
    b: path.as_ref().as_ptr() as usize,
    c: path.as_ref().len(),
    d: O_CREAT | O_DIRECTORY,
  };
  let msg = msg.call(SERVER_REDOX_FS).map_err(|_| Error::new(EIO))?;
  let f = Error::demux(msg.a).map(|handle| File{ handle })?;
  drop(f);
  Ok(())
}

pub fn remove_file<P: AsRef<str>>(path: P) -> Result<()> {
  let msg = Message {
    a: SYS_UNLINK,
    b: path.as_ref().as_ptr() as usize,
    c: path.as_ref().len(),
    d: 0,
  };
  msg.call(SERVER_REDOX_FS).map_err(|_| Error::new(EIO))?;
  Error::demux(msg.a).map(|_| ())
}

pub fn remove_directory<P: AsRef<str>>(path: P) -> Result<()> {
  let msg = Message {
    a: SYS_RMDIR,
    b: path.as_ref().as_ptr() as usize,
    c: path.as_ref().len(),
    d: 0,
  };
  msg.call(SERVER_REDOX_FS).map_err(|_| Error::new(EIO))?;
  Error::demux(msg.a).map(|_| ())
}
