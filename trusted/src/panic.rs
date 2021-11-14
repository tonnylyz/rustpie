#[allow(dead_code)]
static mut PANICKED: bool = false;

#[allow(dead_code)]
pub fn random_panic() {
  unsafe {
    if !PANICKED {
      PANICKED = true;
      panic!("[[RANDOM]][[PANIC]]");
    }
  }
}

#[allow(dead_code)]
static mut PAGEFAULT: bool = false;

#[allow(dead_code)]
pub fn random_page_fault() {
  unsafe {
    if !PAGEFAULT {
      PAGEFAULT = true;
      info!("[[RANDOM]][[PAGEFAULT]]");
      (0xdeafbeef00000000 as *mut u32).read_volatile();
    }
  }
}
