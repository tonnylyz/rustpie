use crate::fs::{Disk, BLOCK_SIZE};
use common::PAGE_SIZE;
use rlibc::memcpy;
use trusted::redoxcall::*;
use trusted::message::Message;

pub struct VirtioClient {
    tid: u16,
}

impl VirtioClient {
    pub fn new() -> VirtioClient {
        let server_tid = microcall::server_tid_wait(common::server::SERVER_VIRTIO_BLK);
        println!("virtio client {}", server_tid);
        VirtioClient {
            tid: server_tid,
        }
    }
}

impl Disk for VirtioClient {
    fn read_at(&mut self, block: u64, buffer: &mut [u8]) -> Result<usize> {
        assert_eq!(buffer.len() % BLOCK_SIZE as usize, 0);
        assert_eq!(buffer.len(), PAGE_SIZE);
        let tmp = trusted::mm::valloc(1);
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

    fn write_at(&mut self, _block: u64, _buffer: &[u8]) -> Result<usize> {
        unimplemented!()
    }

    fn size(&mut self) -> Result<u64> {
        // TODO: obtain real size
        Ok(536870912) // 512MB
    }
}
