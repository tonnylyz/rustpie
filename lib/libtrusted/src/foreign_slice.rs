use crate::loader::{round_up, round_down};
use crate::mm::{valloc, Entry, PageAttribute, default_page_attribute};
use common::PAGE_SIZE;
use microcall::{mem_map, mem_unmap};
use redox::*;

pub struct ForeignSlice {
  pub asid: u16,
  pub slice_start: usize,
  pub slice_len: usize,
  pub local_start: usize,
  page_num: usize,
  local_buf: usize,
}

impl ForeignSlice {
  pub fn new(asid: u16, slice_start: usize, slice_len: usize) -> Result<Self> {
    let page_num = (round_up(slice_start + slice_len, PAGE_SIZE)
      - round_down(slice_start, PAGE_SIZE)) / PAGE_SIZE;
    let local_buf = valloc(page_num) as usize;
    let local_start = slice_start - round_down(slice_start, PAGE_SIZE) + local_buf;

    for i in 0..page_num {
      let src_va = round_down(slice_start, PAGE_SIZE) + i * PAGE_SIZE;
      let dst_va = local_buf + i * PAGE_SIZE;
      mem_map(asid, src_va, 0, dst_va, default_page_attribute()).map_err(|_e| Error::new(EINVAL))?;
    }

    Ok(ForeignSlice {
      asid,
      slice_start,
      slice_len,
      local_start,
      page_num,
      local_buf,
    })
  }

  pub fn local_slice(&self) -> &[u8] {
    unsafe {
      core::slice::from_raw_parts(self.local_start as *const u8, self.slice_len)
    }
  }

  pub fn local_slice_mut(&self) -> &[u8] {
    unsafe {
      core::slice::from_raw_parts_mut(self.local_start as *mut u8, self.slice_len)
    }
  }
}

impl Drop for ForeignSlice {
  fn drop(&mut self) {
    for i in 0..self.page_num {
      let va = self.local_buf + i * PAGE_SIZE;
      let _ = mem_unmap(0, va);
    }
  }
}
