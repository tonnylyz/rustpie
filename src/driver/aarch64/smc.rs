#[cfg(not(any(feature = "shyper", feature = "tx2")))]
pub fn smc_call(x0: u64, x1: u64, x2: u64, x3: u64) -> u64 {
  let r;
  unsafe {
    // NOTE: here use hvc for qemu without `virtualization=on`
    asm!("hvc #0", inlateout("x0") x0 => r, in("x1") x1, in("x2") x2, in("x3") x3);
  }
  r
}

#[cfg(any(feature = "shyper", feature = "tx2"))]
pub fn smc_call(x0: u64, x1: u64, x2: u64, x3: u64) -> u64 {
  let r;
  unsafe {
    // NOTE: here use smc for shyper
    asm!("smc #0", inlateout("x0") x0 => r, in("x1") x1, in("x2") x2, in("x3") x3);
  }
  r
}
