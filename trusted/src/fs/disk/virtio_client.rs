use crate::fs::{Disk, BLOCK_SIZE};
use common::PAGE_SIZE;
use rlibc::memcpy;
use libtrusted::redoxcall::*;
use libtrusted::message::Message;

pub struct VirtioClient {
    tid: u16,
}

impl VirtioClient {
    pub fn new() -> VirtioClient {
        let server_tid = microcall::server_tid_wait(common::server::SERVER_VIRTIO_BLK);
        VirtioClient {
            tid: server_tid,
        }
    }

    fn read_block(&self, block: u64, buffer: &mut [u8]) -> Result<usize> {
        assert_eq!(buffer.len() % BLOCK_SIZE as usize, 0);
        assert_eq!(buffer.len(), PAGE_SIZE);
        let tmp = libtrusted::mm::valloc(1);
        let sector = (block as usize) * 8;
        let count = 8;
        let buf = unsafe { core::slice::from_raw_parts_mut(tmp, PAGE_SIZE) };
        let msg = Message {
            a: sector,
            b: count,
            c: buf.as_mut_ptr() as usize,
            d: 0
        }.call(self.tid);
        println!("[FS] VirtioClient RX {:x?}", msg);
        assert_eq!(msg.a, 0);
        unsafe { memcpy(buffer.as_mut_ptr(), tmp, PAGE_SIZE); }
        Ok(buffer.len())
    }
}

impl Disk for VirtioClient {
    fn read_at(&mut self, block: u64, buffer: &mut [u8]) -> Result<usize> {
        assert_eq!(buffer.len() % BLOCK_SIZE as usize, 0);
        let page_num = buffer.len() / PAGE_SIZE;
        let mut sum = 0;
        for i in 0..page_num {
            sum += self.read_block(block + (i as u64), &mut buffer[(PAGE_SIZE * i)..(PAGE_SIZE * (i + 1))])?;
        }
        Ok(sum)
    }

    fn write_at(&mut self, _block: u64, _buffer: &[u8]) -> Result<usize> {
        unimplemented!()
    }

    fn size(&mut self) -> Result<u64> {
        // TODO: obtain real size
        Ok(536870912) // 512MB
    }
}
