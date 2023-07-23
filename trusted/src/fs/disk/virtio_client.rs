use rpabi::PAGE_SIZE;

use crate::libtrusted::mm::{virtual_alloc, virtual_free};
use rpsyscall::message::Message;
use syscall::{Result, Error, EIO};
use redoxfs::{BLOCK_SIZE, Disk};
pub struct VirtioClient;

impl VirtioClient {
  pub fn new() -> VirtioClient {
    VirtioClient
  }

  fn read_block_unaligned(&self, block: u64, buffer: &mut [u8]) -> Result<usize> {
    assert_eq!(buffer.len(), PAGE_SIZE);
    let va_tmp = virtual_alloc(1, true).unwrap();
    let aligned_buffer = unsafe { core::slice::from_raw_parts_mut(va_tmp as *mut u8, PAGE_SIZE) };
    let read = self.read_block_aligned(block, aligned_buffer)?;
    for i in 0..PAGE_SIZE {
      buffer[i] = aligned_buffer[i];
    }
    virtual_free(va_tmp, 1);
    Ok(read)
  }

  fn read_block_aligned(&self, block: u64, buffer: &mut [u8]) -> Result<usize> {
    assert_eq!(buffer.len(), PAGE_SIZE);
    let msg = Message {
      a: (block as usize) * 8,
      b: 8,
      c: buffer.as_mut_ptr() as usize,
      d: rpservapi::blk::action::READ,
    }.call(rpabi::server::SERVER_BLK).map_err(|_| Error::new(EIO))?;
    if msg.a == rpservapi::blk::result::OK {
      Ok(buffer.len())
    } else {
      Err(Error::new(EIO))
    }
  }

  fn write_block_unaligned(&self, block: u64, buffer: &[u8]) -> Result<usize> {
    assert_eq!(buffer.len(), PAGE_SIZE);
    let va_tmp = virtual_alloc(1, true).unwrap();
    let aligned_buffer = unsafe { core::slice::from_raw_parts_mut(va_tmp as *mut u8, PAGE_SIZE) };
    for i in 0..PAGE_SIZE {
      aligned_buffer[i] = buffer[i];
    }
    let read = self.write_block_aligned(block, aligned_buffer)?;
    virtual_free(va_tmp, 1);
    Ok(read)
  }

  fn write_block_aligned(&self, block: u64, buffer: &[u8]) -> Result<usize> {
    assert_eq!(buffer.len(), PAGE_SIZE);
    let msg = Message {
      a: (block as usize) * 8,
      b: 8,
      c: buffer.as_ptr() as usize,
      d: rpservapi::blk::action::WRITE,
    }.call(rpabi::server::SERVER_BLK).map_err(|_| Error::new(EIO))?;
    if msg.a == rpservapi::blk::result::OK {
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

  fn write_at(&mut self, block: u64, buffer: &[u8]) -> Result<usize> {
    assert_eq!(buffer.len() % BLOCK_SIZE as usize, 0);
    let page_num = buffer.len() / PAGE_SIZE;
    let mut sum = 0;
    for i in 0..page_num {
      if buffer.as_ptr() as usize % PAGE_SIZE == 0 {
        sum += self.write_block_aligned(block + (i as u64), &buffer[(PAGE_SIZE * i)..(PAGE_SIZE * (i + 1))])?;
      } else {
        sum += self.write_block_unaligned(block + (i as u64), &buffer[(PAGE_SIZE * i)..(PAGE_SIZE * (i + 1))])?;
      }
    }
    Ok(sum)
  }

  fn size(&mut self) -> Result<u64> {
    let msg = Message {
      a: 0,
      b: 0,
      c: 0,
      d: rpservapi::blk::action::SIZE,
    };
    let msg = msg.call(rpabi::server::SERVER_BLK).map_err(|_| Error::new(EIO))?;
    if msg.a == rpservapi::blk::result::OK {
      Err(Error::new(EIO))
    } else {
      Ok(msg.a as u64)
    }
  }
}
