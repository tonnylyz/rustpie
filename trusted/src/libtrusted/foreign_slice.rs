use rpabi::PAGE_SIZE;

use rpsyscall::mem_map;
use crate::libtrusted::loader::round_up;
use crate::libtrusted::loader::round_down;
use crate::libtrusted::mm::{default_page_attribute, virtual_alloc, virtual_free};

pub struct ForeignSlice {
  pub asid: u16,
  pub slice_start: usize,
  pub slice_len: usize,
  pub local_start: usize,
  page_num: usize,
  local_buf: usize,
}

impl ForeignSlice {
  pub fn new(asid: u16, slice_start: usize, slice_len: usize) -> Result<Self, isize> {
    let page_num = (round_up(slice_start + slice_len, PAGE_SIZE)
      - round_down(slice_start, PAGE_SIZE)) / PAGE_SIZE;
    let local_buf = virtual_alloc(page_num, false).unwrap() as usize;
    let local_start = slice_start - round_down(slice_start, PAGE_SIZE) + local_buf;

    for i in 0..page_num {
      let src_va = round_down(slice_start, PAGE_SIZE) + i * PAGE_SIZE;
      let dst_va = local_buf + i * PAGE_SIZE;
      mem_map(asid, src_va, 0, dst_va, default_page_attribute()).map_err(|_e| -1isize)?;
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

  #[allow(dead_code)]
  pub fn local_slice_mut(&self) -> &[u8] {
    unsafe {
      core::slice::from_raw_parts_mut(self.local_start as *mut u8, self.slice_len)
    }
  }
}

impl Drop for ForeignSlice {
  fn drop(&mut self) {
    virtual_free(self.local_buf, self.page_num);
  }
}
