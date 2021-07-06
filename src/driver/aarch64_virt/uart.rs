use crate::Address;

pub fn init() {}

const UART_BASE: usize = 0x900_0000;

pub fn putc(c: u8) {
  unsafe {
    let ptr = UART_BASE.pa2kva();
    let val = c as u32;
    (ptr as *mut u32).write_volatile(val);
  }
}
