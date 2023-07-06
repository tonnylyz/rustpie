use rpabi::syscall::error::{ERROR_INVARG, ERROR_OOM};
use xmas_elf::*;

use crate::arch::{PAGE_SIZE, PageTable};
use crate::lib::traits::Address;
use crate::mm::Frame;
use crate::mm::page_table::{EntryAttribute, PageTableEntryAttrTrait, PageTableTrait};
use crate::util::round_up;

pub type Error = usize;

pub fn load(src: &'static [u8], page_table: &PageTable) -> Result<usize, Error> {
  if let Ok(elf) = ElfFile::new(src) {
    let entry_point = elf.header.pt2.entry_point() as usize;
    for ph in elf.program_iter() {
      if let Ok(program::Type::Load) = ph.get_type() {
        /* Ignore types other than `Load` */
      } else {
        continue;
      }

      let va = ph.virtual_addr() as usize;
      if va % PAGE_SIZE != 0 {
        warn!("ignore unaligned program@{:016x}", va);
        continue;
      }
      if va == 0 {
        warn!("ignore program@{:016x}", va);
        continue;
      }

      let mem_page_num = round_up(ph.mem_size() as usize, PAGE_SIZE) / PAGE_SIZE;
      let file_page_num = round_up(ph.file_size() as usize, PAGE_SIZE) / PAGE_SIZE;
      let pa = (src.as_ptr() as usize + ph.offset() as usize).kva2pa();

      trace!("map {:08x} to {:08x} len {:x}/{:x} {}", va, pa, file_page_num * PAGE_SIZE, mem_page_num * PAGE_SIZE, ph.flags());


      if !(ph.flags().is_read()) {
        warn!("ignore not readable program@{:016x}", va);
        continue;
      }
      let attr = if ph.flags().is_execute() {
        // R E
        EntryAttribute::user_executable()
      } else {
        if ph.flags().is_write() {
          // RW
          EntryAttribute::user_data()
        } else {
          // R
          EntryAttribute::user_data()
        }
      };

      for i in 0..file_page_num {
        page_table.map(va + i * PAGE_SIZE, pa + i * PAGE_SIZE, attr)?;
      }
      for i in file_page_num..mem_page_num {
        let frame = crate::mm::page_pool::page_alloc().map_err(|_| ERROR_OOM)?;
        frame.zero();
        page_table.insert_page(va + i * PAGE_SIZE, Frame::from(frame), attr)?;
      }
    }
    Ok(entry_point)
  } else {
    Err(ERROR_INVARG)
  }
}
