use crate::arch::{PAGE_SIZE, PageTable};
use crate::mm::page_table::{EntryAttribute, PageTableEntryAttrTrait, PageTableTrait};
use crate::util::round_up;
use xmas_elf::*;

pub enum Error {
  ElfHeaderError,
  ElfPageTableError,
}

impl core::convert::From<crate::mm::page_table::Error> for Error {
  fn from(_: crate::mm::page_table::Error) -> Self {
    Error::ElfPageTableError
  }
}

#[inline(always)]
fn copy(src: &[u8], src_offset: usize, dst: &mut [u8], dst_offset: usize, length: usize) {
  for i in 0..length {
    dst[dst_offset + i] = src[src_offset + i];
  }
}

pub fn load(src: &'static [u8], page_table: &PageTable) -> Result<usize, Error> {

  if let Ok(elf) = ElfFile::new(src) {
    let entry_point = elf.header.pt2.entry_point() as usize;
    for program_header in elf.program_iter() {
      if let Ok(program::Type::Load) = program_header.get_type() {
        /* Ignore types other than `Load` */
      } else {
        continue;
      }
      let va = program_header.virtual_addr() as usize;
      let file_size = program_header.file_size() as usize;
      let file_end = va + file_size;
      let mem_size = program_header.mem_size() as usize;
      let mem_end = va + mem_size;

      if va % PAGE_SIZE != 0 {
        warn!("ignore unaligned section@{:016x}", va);
        continue;
      }


      let mut i = va;
      while i < round_up(file_end, PAGE_SIZE) {
        let frame = crate::mm::page_pool::alloc();
        frame.zero();
        let frame_slice = frame.as_mut_slice();
        let uf = crate::mm::UserFrame::new_memory(frame);
        // trace!("mapping {:016x} -> {:08x}", i, uf.pa());
        page_table.insert_page(i, uf, EntryAttribute::user_default())?;
        let offset = program_header.offset() as usize + (i - va);
        copy(src, offset, frame_slice, 0, PAGE_SIZE);

        i += PAGE_SIZE;
      }
      while i < round_up(mem_end, PAGE_SIZE) {
        let frame = crate::mm::page_pool::alloc();
        frame.zero();
        let uf = crate::mm::UserFrame::new_memory(frame);
        // trace!("allocating {:016x} -> {:08x}", i, uf.pa());
        page_table.insert_page(i, uf, EntryAttribute::user_default())?;

        i += PAGE_SIZE;
      }
    }
    Ok(entry_point)
  } else {
    Err(Error::ElfHeaderError)
  }
}