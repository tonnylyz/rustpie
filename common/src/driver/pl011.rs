use tock_registers::register_structs;
use tock_registers::registers::*;
use tock_registers::interfaces::{Readable, Writeable};

register_structs! {
  #[allow(non_snake_case)]
  pub Pl011MmioBlock {
    (0x000 => pub Data: ReadWrite<u32>),
    (0x004 => pub RecvStatusErrClr: ReadWrite<u32>),
    (0x008 => _reserved_0),
    (0x018 => pub Flag: ReadOnly<u32>),
    (0x01c => _reserved_1),
    (0x020 => pub IrDALowPower: ReadWrite<u32>),
    (0x024 => pub IntBaudRate: ReadWrite<u32>),
    (0x028 => pub FracBaudRate: ReadWrite<u32>),
    (0x02c => pub LineControl: ReadWrite<u32>),
    (0x030 => pub Control: ReadWrite<u32>),
    (0x034 => pub IntFIFOLevel: ReadWrite<u32>),
    (0x038 => pub IntMaskSetClr: ReadWrite<u32>),
    (0x03c => pub RawIntStatus: ReadOnly<u32>),
    (0x040 => pub MaskedIntStatus: ReadOnly<u32>),
    (0x044 => pub IntClear: WriteOnly<u32>),
    (0x048 => pub DmaControl: ReadWrite<u32>),
    (0x04c => _reserved_2),
    (0x1000 => @END),
  }
}

pub struct Pl011Mmio {
  base_addr: usize,
}

impl core::ops::Deref for Pl011Mmio {
  type Target = Pl011MmioBlock;

  fn deref(&self) -> &Self::Target {
    unsafe { &*self.ptr() }
  }
}

impl Pl011Mmio {
  pub const fn new(base_addr: usize) -> Self { Pl011Mmio { base_addr } }
  fn ptr(&self) -> *const Pl011MmioBlock { self.base_addr as *const _ }
}
