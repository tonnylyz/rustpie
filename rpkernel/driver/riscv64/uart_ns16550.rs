use tock_registers::interfaces::{Readable, Writeable};
use hardware::ns16550::*;

const NS16550_MMIO_BASE: usize = 0xffff_ffff_0000_0000 + 0x1000_0000;

static NS16550_MMIO: Ns16550Mmio = Ns16550Mmio::new(NS16550_MMIO_BASE);

pub fn init() {
  let uart = &NS16550_MMIO;
  uart.ISR_FCR
    .write(ISR_FCR::EN_FIFO::Mode16550);
}

fn send(c: u8) {
  let uart = &NS16550_MMIO;
  while !uart.LSR.is_set(LSR::THRE) {
    // Wait until it is possible to write data.
  }
  uart.RHR_THR_DLL.set(c);
}

pub fn putc(c: u8) {
  if c == b'\n' {
    send(b'\r');
  }
  send(c);
}

pub fn getc() -> Option<u8> {
  let uart = &NS16550_MMIO;
  if uart.LSR.is_set(LSR::RDR) {
    Some(uart.RHR_THR_DLL.get() as u8)
  } else {
    None
  }
}
