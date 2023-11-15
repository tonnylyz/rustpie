use rpabi::string::memset;

#[no_mangle]
#[link_section = ".text.start"]
pub extern "C" fn _start(boot_data: usize) -> ! {
  let stack_top = crate::kernel::stack::stack_of_core(0);
  extern "C" {
    fn BSS_START();
    fn BSS_END();
  }
  let bss_start = BSS_START as usize;
  let bss_end = BSS_END as usize;
  let bss_size = bss_end - bss_start;
  unsafe {
    memset(bss_start as *mut u8, 0, bss_size);
    core::arch::asm!("mov rsp, {}", in(reg) stack_top);
  }
  crate::main(0, boot_data);
}
