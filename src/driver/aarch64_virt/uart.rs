use crate::arch::Address;

pub fn init() {

}

const UART_BASE: usize = 0x900_0000;

pub fn putc(c: u8) {
    unsafe { crate::driver::mmio::write_word(UART_BASE.pa2kva(), c as u32); }
}