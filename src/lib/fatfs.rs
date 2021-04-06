use core_io as io;
use io::{Read, SeekFrom};
use io::prelude::*;

use crate::lib::{round_down, round_up};

struct Disk {
  pointer: usize,
  size: usize,
}

impl Read for Disk {
  fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
    let sector = round_down(self.pointer, 512) / 512;
    let offset = self.pointer - round_down(self.pointer, 512);
    let count = round_up(offset + buf.len(), 512) / 512;
    assert!(count <= 8);
    let frame = crate::mm::page_pool::alloc();
    println!("read sector {} count {} offset {} buf.len {} pointer {}", sector, count, offset, buf.len(), self.pointer);
    crate::driver::common::virtio_blk::read(sector, count, frame.kva());
    for i in 0..buf.len() {
      buf[i] = frame.as_slice()[offset + i];
    }
    self.pointer += buf.len();
    Ok(buf.len())
  }
}

impl Write for Disk {
  fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
    println!("write dropped");
    Ok(0)
  }
  fn flush(&mut self) -> io::Result<()> {
    Ok(())
  }
}

impl Seek for Disk {
  fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
    match pos {
      SeekFrom::Start(u) => {
        self.pointer = u as usize;
      }
      SeekFrom::End(i) => {
        self.pointer = self.size - (i as usize);
      }
      SeekFrom::Current(i) => {
        self.pointer += i as usize;
      }
    }
    Ok(self.pointer as u64)
  }
}

// pub fn init() {
//
//   let mut disk = Disk{ pointer: 0, size: 536870912 };
//   let fs = fatfs::FileSystem::new(&mut disk, fatfs::FsOptions::new()).unwrap();
//   let root_dir = fs.root_dir();
//   let mut file = root_dir.open_file("hello.txt").unwrap();
//   let mut buf = [0u8; 13];
//   let len = file.seek(SeekFrom::End(0)).unwrap();
//   println!("len {}", len);
//
//   file.seek(SeekFrom::Start(0));
//   file.read_exact(&mut buf).unwrap();
//   println!("{}", buf.map(|u| u as char).iter().collect::<String>());
// }