use crate::fs::{File, SeekFrom};
use common::PAGE_SIZE;
use crate::mm::EntryLike;
use xmas_elf::program::SegmentData;

#[inline(always)]
pub fn round_up(addr: usize, n: usize) -> usize {
  (addr + n - 1) & !(n - 1)
}
#[inline(always)]
pub fn round_down(addr: usize, n: usize) -> usize {
  addr & !(n - 1)
}

pub fn spawn<P: AsRef<str>>(path: P, arg: usize) -> Result<u16, &'static str> {
  let asid = microcall::address_space_alloc().map_err(|e| "address_space_alloc failed")?;
  let mut f = File::open(path).map_err(|e| "open file failed")?;
  let file_size = f.seek(SeekFrom::End(0)).map_err(|e| "seek end failed")? as usize;
  let page_num = round_up(file_size, PAGE_SIZE) / PAGE_SIZE;
  let buf = crate::mm::valloc(page_num);
  let buf = unsafe { core::slice::from_raw_parts_mut(buf, file_size) };
  f.seek(SeekFrom::Start(0)).map_err(|e| "seek start failed")? as usize;
  let read = f.read(buf).map_err(|e| "read failed")?;
  // println!("read {} byte", read);
  // for i in 0..read {
  //   print!("{:02x} ", buf[i]);
  // }
  let elf = xmas_elf::ElfFile::new(buf)?;
  let entry_point = elf.header.pt2.entry_point() as usize;
  for ph in elf.program_iter() {
    if let Ok(xmas_elf::program::Type::Load) = ph.get_type() {

    } else {
      continue;
    }
    let va_start = ph.virtual_addr() as usize;
    if va_start % PAGE_SIZE != 0 {
      continue;
    }
    let va_end = round_up(va_start + ph.mem_size() as usize, PAGE_SIZE);
    let mut va = round_down(va_start, PAGE_SIZE);
    let va_tmp = crate::mm::virtual_page_alloc(1);
    while va < va_end {
      microcall::mem_alloc(asid, va, crate::mm::Entry::default().attribute()).map_err(|e| "out of memory")?;
      // println!("alloc @{:016x}", va);
      unsafe {
        if va < va_start + ph.file_size() as usize {
          microcall::mem_map(asid, va, 0, va_tmp, crate::mm::Entry::default().attribute())
            .map_err(|e| "mem_map failed");
          let va_slice = core::slice::from_raw_parts_mut(va_tmp as *mut u8, PAGE_SIZE);
          let offset = ph.offset() as usize + (va - va_start);
          // println!("offset {:x}", offset);
          for i in 0..PAGE_SIZE {
            va_slice[i] = buf[offset + i];
          }
          // println!("copy into {:016x}", va);
          microcall::mem_unmap(0, va_tmp).map_err(|e| "mem_unmap failed")?;
        }
      }

      va += PAGE_SIZE;
    }
  }
  microcall::mem_alloc(asid, common::CONFIG_USER_STACK_TOP - PAGE_SIZE, crate::mm::Entry::default().attribute());
  let tid = microcall::thread_alloc(asid, entry_point, common::CONFIG_USER_STACK_TOP, arg).map_err(|e| "thread alloc failed")?;
  // println!("[LOADER] spawn asid {} tid {}", asid, tid);
  microcall::thread_set_status(tid, common::thread::THREAD_STATUS_RUNNABLE);

  Ok(asid)
}
