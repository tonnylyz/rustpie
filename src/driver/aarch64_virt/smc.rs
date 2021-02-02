
#[no_mangle]
pub fn smc_call(x0: u64, x1: u64, x2: u64, x3: u64) -> u64 {
  let r;
  unsafe {
    llvm_asm!("smc #0"
        : "={x0}" (r)
        : "{x0}" (x0), "{x1}" (x1), "{x2}" (x2), "{x3}" (x3)
        : "memory"
        : "volatile");
  }
  r
}
