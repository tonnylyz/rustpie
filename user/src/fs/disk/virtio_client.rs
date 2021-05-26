use crate::fs::{Disk, BLOCK_SIZE};
use crate::syscall::*;
use crate::constants::PAGE_SIZE;
use crate::itc::ItcMessage;
use crate::mem::valloc;
use rlibc::memcpy;
use crate::root::Server::VirtioBlk;

pub struct VirtioClient;

impl VirtioClient {
    pub fn new() -> VirtioClient {
        VirtioClient
    }
}

impl Disk for VirtioClient {
    fn read_at(&mut self, block: u64, buffer: &mut [u8]) -> Result<usize> {
        assert_eq!(buffer.len() % BLOCK_SIZE as usize, 0);
        assert_eq!(buffer.len(), PAGE_SIZE);
        let tmp =valloc(1);
        let sector = (block as usize) * 8;
        let count = 8;
        let buf = unsafe { core::slice::from_raw_parts_mut(tmp, PAGE_SIZE) };
        ItcMessage {
            a: sector,
            b: count,
            c: buf.as_mut_ptr() as usize,
            d: 0
        }.send_to_server(VirtioBlk);
        let msg = ItcMessage::receive();
        // assert_eq!(msg.0, self.server_tid);
        println!("[FS] VirtoClient RX {:x?}", msg);
        assert_eq!(msg.1.a, 0);
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
