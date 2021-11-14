use common::PAGE_SIZE;

use fs::{File, SeekFrom};

use crate::mm::{default_page_attribute, virtual_alloc, virtual_free};

#[inline(always)]
pub fn round_up(addr: usize, n: usize) -> usize {
  (addr + n - 1) & !(n - 1)
}

#[inline(always)]
pub fn round_down(addr: usize, n: usize) -> usize {
  addr & !(n - 1)
}

pub fn spawn<P: AsRef<str>>(cmd: P) -> Result<(u16, usize), &'static str> {
  let mut iter = cmd.as_ref().trim().split_ascii_whitespace();
  if let Some(bin) = iter.next() {
    let asid = microcall::address_space_alloc().map_err(|_e| "address_space_alloc failed")?;
    let mut f = File::open(bin).map_err(|e| {
      error!("spawn open file failed");
      e.text()
    })?;
    let file_size = f.seek(SeekFrom::End(0)).map_err(|e| {
      error!("spawn seek end file failed");
      e.text()
    })? as usize;
    let page_num = round_up(file_size, PAGE_SIZE) / PAGE_SIZE;
    // info!("spawn elf size {} pages", page_num);
    f.seek(SeekFrom::Start(0)).map_err(|e| {
      error!("spawn seek start file failed");
      e.text()
    })? as usize;
    let buf = virtual_alloc(page_num, true).unwrap() as *mut u8;
    let buf = unsafe { core::slice::from_raw_parts_mut(buf, file_size) };
    let _read = f.read(buf).map_err(|e| {
      error!("spawn read file failed");
      e.text()
    })?;
    let elf = xmas_elf::ElfFile::new(buf)?;
    let entry_point = elf.header.pt2.entry_point() as usize;
    let va_tmp = virtual_alloc(1, false).unwrap();
    for ph in elf.program_iter() {
      if let Ok(xmas_elf::program::Type::Load) = ph.get_type() {} else {
        continue;
      }
      let va_start = ph.virtual_addr() as usize;
      if va_start % PAGE_SIZE != 0 {
        continue;
      }
      let va_end = round_up(va_start + ph.mem_size() as usize, PAGE_SIZE);
      let mut va = round_down(va_start, PAGE_SIZE);
      while va < va_end {
        microcall::mem_alloc(asid, va, crate::mm::default_page_attribute()).map_err(|_e| "out of memory")?;
        // println!("alloc @{:016x}", va);
        unsafe {
          if va < va_start + ph.file_size() as usize {
            microcall::mem_map(asid, va, 0, va_tmp, crate::mm::default_page_attribute())
              .map_err(|_e| "mem_map failed")?;
            let va_slice = core::slice::from_raw_parts_mut(va_tmp as *mut u8, PAGE_SIZE);
            let offset = ph.offset() as usize + (va - va_start);
            // println!("offset {:x}", offset);
            for i in 0..PAGE_SIZE {
              va_slice[i] = buf[offset + i];
            }
            // println!("copy into {:016x}", va);
            microcall::mem_unmap(0, va_tmp).map_err(|_e| "mem_unmap failed")?;
          }
        }

        va += PAGE_SIZE;
      }
    }
    virtual_free(buf.as_ptr() as usize, page_num);
    microcall::mem_alloc(asid, common::CONFIG_USER_STACK_TOP - PAGE_SIZE, crate::mm::default_page_attribute()).map_err(|_e| "mem_alloc failed")?;
    microcall::mem_map(asid, common::CONFIG_USER_STACK_TOP - PAGE_SIZE, 0, va_tmp, default_page_attribute()).map_err(|_e| "mem_map failed")?;
    let va_slice = unsafe { core::slice::from_raw_parts_mut(va_tmp as *mut u8, PAGE_SIZE) };
    let mut index = 0;
    loop {
      if let Some(arg) = iter.next() {
        for i in 0..arg.len() {
          va_slice[index] = arg.as_bytes()[i];
          index += 1;
        }
        va_slice[index] = b' ';
        index += 1;
      } else {
        break;
      }
    }
    va_slice[index] = 0;
    index += 1;
    for i in 0..index {
      va_slice[PAGE_SIZE - index + i] = va_slice[i];
    }
    virtual_free(va_tmp, 1);

    let tid = microcall::thread_alloc(asid, entry_point, common::CONFIG_USER_STACK_TOP - round_up(index, 16), common::CONFIG_USER_STACK_TOP - index).map_err(|_e| "thread alloc failed")?;
    // println!("[LOADER] spawn asid {} tid {}", asid, tid);

    Ok((asid, tid))
  } else {
    Err("cmd does not has bin")
  }
}
