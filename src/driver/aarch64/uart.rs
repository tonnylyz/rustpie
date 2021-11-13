use tock_registers::interfaces::{Readable, Writeable};
use common::driver::pl011::Pl011Mmio;


pub fn init() {}

const UART_BASE: usize = 0xFFFF_FF80_0000_0000 + 0x900_0000;

static PL011_MMIO: Pl011Mmio = Pl011Mmio::new(UART_BASE);

const UART_FR_RXFF: u32 = 1 << 4;
const UART_FR_TXFF: u32 = 1 << 5;

pub fn putc(c: u8) {
  if c == b'\n' {
    putc(b'\r');
  }
  let pl011 = &PL011_MMIO;
  loop {
    if pl011.Flag.get() & UART_FR_TXFF == 0 {
      break;
    }
  }
  pl011.Data.set(c as u32);
}

pub fn getc() -> Option<u8> {
  let pl011 = &PL011_MMIO;
  if pl011.Flag.get() & UART_FR_RXFF == 0 {
    Some((pl011.Data.get() & 0xff) as u8)
  } else {
    None
  }
}