use tock_registers::registers::*;
use tock_registers::interfaces::{Readable, Writeable};

use common::driver::pl011::Pl011Mmio;

const PL011_MMIO_BASE: usize = 0x8_0000_0000 + 0x900_0000;

static PL011_MMIO: Pl011Mmio = Pl011Mmio::new(PL011_MMIO_BASE);


pub fn irq() {
  let pl011 = &PL011_MMIO;
  let status = pl011.RawIntStatus.get();
  if status != 0 {
    pl011.IntClear.set(status);
  }
  loop {
    if pl011.Flag.get() & 0b1_0000 != 0 { // RXFE
      break;
    } else {
      let c = pl011.Data.get() & 0xff;
      let mut buf = super::buffer().lock();
      buf.push_back(c as u8);
    }
  }
}

pub fn enable_irq() {
  let pl011 = &PL011_MMIO;
  pl011.Control.set(0x301);
  pl011.IntClear.set(0b111_1111_1111);
  pl011.IntMaskSetClr.set(0b1_0000);
}
