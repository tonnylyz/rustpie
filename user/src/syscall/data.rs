use core::ops::{Deref, DerefMut};
use super::*;
// from redox_syscall data.rs

#[derive(Copy, Clone, Debug, Default)]
#[repr(C)]
pub struct Map {
  /// The offset inside the file that is being mapped.
  pub offset: usize,

  /// The size of the memory map.
  pub size: usize,

  /// Contains both prot and map flags.
  pub flags: MapFlags,

  /// Functions as a hint to where in the virtual address space of the running process, to place
  /// the memory map. If [`MapFlags::MAP_FIXED`] is set, then this address must be the address to
  /// map to.
  pub address: usize,
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
#[repr(C)]
pub struct Stat {
  pub st_dev: u64,
  pub st_ino: u64,
  pub st_mode: u16,
  pub st_nlink: u32,
  pub st_uid: u32,
  pub st_gid: u32,
  pub st_size: u64,
  pub st_blksize: u32,
  pub st_blocks: u64,
  pub st_mtime: u64,
  pub st_mtime_nsec: u32,
  pub st_atime: u64,
  pub st_atime_nsec: u32,
  pub st_ctime: u64,
  pub st_ctime_nsec: u32,
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
#[repr(C)]
pub struct TimeSpec {
  pub tv_sec: i64,
  pub tv_nsec: i32,
}

#[derive(Copy, Clone, Debug, Default)]
#[repr(C)]
pub struct Packet {
  pub id: u64,
  pub pid: usize,
  pub uid: u32,
  pub gid: u32,
  pub a: usize,
  pub b: usize,
  pub c: usize,
  pub d: usize
}

impl Deref for Packet {
  type Target = [u8];
  fn deref(&self) -> &[u8] {
    unsafe {
      slice::from_raw_parts(self as *const Packet as *const u8, mem::size_of::<Packet>())
    }
  }
}

impl DerefMut for Packet {
  fn deref_mut(&mut self) -> &mut [u8] {
    unsafe {
      slice::from_raw_parts_mut(self as *mut Packet as *mut u8, mem::size_of::<Packet>())
    }
  }
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
#[repr(C)]
pub struct StatVfs {
  pub f_bsize: u32,
  pub f_blocks: u64,
  pub f_bfree: u64,
  pub f_bavail: u64,
}

#[derive(Copy, Clone, Debug, Default)]
#[repr(C)]
pub struct OldMap {
  pub offset: usize,
  pub size: usize,
  pub flags: MapFlags,
}
