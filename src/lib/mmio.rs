#[inline(always)]
#[allow(dead_code)]
pub unsafe fn read_dword(ptr: usize) -> u64 {
  (ptr as *mut u64).read_volatile()
}

#[inline(always)]
#[allow(dead_code)]
pub unsafe fn read_word(ptr: usize) -> u32 {
  (ptr as *mut u32).read_volatile()
}

#[inline(always)]
#[allow(dead_code)]
pub unsafe fn read_byte(ptr: usize) -> u8 {
  (ptr as *mut u8).read_volatile()
}

#[inline(always)]
#[allow(dead_code)]
pub unsafe fn write_dword(ptr: usize, val: u64) {
  (ptr as *mut u64).write_volatile(val);
}

#[inline(always)]
#[allow(dead_code)]
pub unsafe fn write_word(ptr: usize, val: u32) {
  (ptr as *mut u32).write_volatile(val);
}

#[inline(always)]
#[allow(dead_code)]
pub unsafe fn write_byte(ptr: usize, val: u8) {
  (ptr as *mut u8).write_volatile(val);
}
