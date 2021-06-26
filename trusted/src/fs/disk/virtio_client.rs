use crate::fs::{Disk, BLOCK_SIZE};
use common::PAGE_SIZE;
use rlibc::memcpy;
use redox::*;
use microcall::message::Message;
use libtrusted::mm::{Entry, EntryLike};

pub struct VirtioClient;

impl VirtioClient {
  pub fn new() -> VirtioClient {
    VirtioClient
  }

  fn read_block_unaligned(&self, block: u64, buffer: &mut [u8]) -> Result<usize> {
    assert_eq!(buffer.len(), PAGE_SIZE);
    let va_tmp = libtrusted::mm::virtual_page_alloc(1);
    microcall::mem_alloc(0, va_tmp, Entry::default().attribute());
    let aligned_buffer = unsafe { core::slice::from_raw_parts_mut(va_tmp as *mut u8, PAGE_SIZE) };
    let read = self.read_block_aligned(block, aligned_buffer)?;
    for i in 0..PAGE_SIZE {
      buffer[i] = aligned_buffer[i];
    }
    microcall::mem_unmap(0, va_tmp);
    Ok(read)
  }

  fn read_block_aligned(&self, block: u64, buffer: &mut [u8]) -> Result<usize> {
    assert_eq!(buffer.len(), PAGE_SIZE);
    let msg = Message {
      a: (block as usize) * 8,
      b: 8,
      c: buffer.as_mut_ptr() as usize,
      d: 0
    }.call(common::server::SERVER_VIRTIO_BLK).map_err(|_| Error::new(EIO))?;
    if msg.a == 0 {
      Ok(buffer.len())
    } else {
      Err(Error::new(EIO))
    }
  }
}

impl Disk for VirtioClient {
  fn read_at(&mut self, block: u64, buffer: &mut [u8]) -> Result<usize> {
    // trace!("start {:016x} len {:016x}", buffer.as_ptr() as usize, buffer.len());
    assert_eq!(buffer.len() % BLOCK_SIZE as usize, 0);
    let page_num = buffer.len() / PAGE_SIZE;
    let mut sum = 0;
    for i in 0..page_num {
      if buffer.as_ptr() as usize % PAGE_SIZE == 0 {
        sum += self.read_block_aligned(block + (i as u64), &mut buffer[(PAGE_SIZE * i)..(PAGE_SIZE * (i + 1))])?;
      } else {
        sum += self.read_block_unaligned(block + (i as u64), &mut buffer[(PAGE_SIZE * i)..(PAGE_SIZE * (i + 1))])?;
      }
    }
    Ok(sum)
  }

  fn write_at(&mut self, _block: u64, _buffer: &[u8]) -> Result<usize> {
    unimplemented!()
  }

  fn size(&mut self) -> Result<u64> {
    let msg = Message {
      a: 0,
      b: 0,
      c: 0,
      d: 2
    };
    let msg = msg.call(common::server::SERVER_VIRTIO_BLK).map_err(|_| Error::new(EIO))?;
    if msg.a == 0 {
      Err(Error::new(EIO))
    } else {
      Ok(msg.a as u64)
    }
  }
}
