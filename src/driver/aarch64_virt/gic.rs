use crate::arch::Address;

const GIC_INTERRUPT_NUM: usize = 1024;
const GIC_SGI_NUM: usize = 16;
const GIC_1_BIT_NUM: usize = GIC_INTERRUPT_NUM / 32;
const GIC_8_BIT_NUM: usize = GIC_INTERRUPT_NUM * 8 / 32;
const GIC_2_BIT_NUM: usize = GIC_INTERRUPT_NUM * 2 / 32;

const GICD_BASE: usize = 0x08000000;
const GICC_BASE: usize = 0x08010000;

#[repr(packed)]
struct GicDistributor {
  control: u32,
  typer: u32,
  iid: u32,
  _reserved_0: [u8; 0x80 - 0x0c],
  group: [u32; GIC_1_BIT_NUM],
  set_enable: [u32; GIC_1_BIT_NUM],
  clear_enable: [u32; GIC_1_BIT_NUM],
  set_pend: [u32; GIC_1_BIT_NUM],
  clear_pend: [u32; GIC_1_BIT_NUM],
  set_active: [u32; GIC_1_BIT_NUM],
  clear_active: [u32; GIC_1_BIT_NUM],
  priority: [u32; GIC_8_BIT_NUM],
  target: [u32; GIC_8_BIT_NUM],
  config: [u32; GIC_2_BIT_NUM],
  _reserved_1: [u8; 0xe00 - 0xd00],
  nsac: [u32; GIC_2_BIT_NUM],
  sgi: u32,
  _reserved_2: [u8; 0xf10 - 0xf04],
  clear_pend_sgi: [u32; GIC_SGI_NUM * 8 / 32],
  set_pend_sgi: [u32; GIC_SGI_NUM * 8 / 32],
}

#[repr(packed)]
struct GicCpuInterface {
  control: u32,
  pmr: u32,
  bpr: u32,
  int_ack: u32,
  end_of_int: u32,
  running_priority: u32,
  highest_pending_priority_int: u32,
  abpr: u32,
  aiar: u32,
  aeoir: u32,
  ahppir: u32,
  _reserved_0: [u8; 0xd0 - 0x2c],
  apr: [u32; 4],
  nsapr: [u32; 4],
  _reserved_1: [u8; 0xfc - 0xf0],
  iid: u32,
  _reserved_2: [u8; 0x1000 - 0x100],
  dir: u32,
}

impl GicCpuInterface {
  fn init(&mut self) {
    self.pmr = u32::max_value();
    self.control |= 1 | (1 << 9);
  }
}

impl GicDistributor {
  fn init(&mut self) {
    let max_spi = (self.typer & 0b11111) * 32 + 1;
    for i in 1usize..(max_spi as usize / 32) {
      self.clear_enable[i] = u32::max_value();
      self.clear_pend[i] = u32::max_value();
      self.clear_active[i] = u32::max_value();
    }
    for i in 8usize..(max_spi as usize * 8 / 32) {
      self.priority[i] = u32::max_value();
      self.target[i] = u32::max_value();
    }
    self.control |= 1;
  }

  fn init_per_core(&mut self) {
    self.clear_enable[0] = u32::max_value();
    self.clear_pend[0] = u32::max_value();
    self.clear_active[0] = u32::max_value();
    for i in 0..4 {
      self.clear_pend_sgi[i] = u32::max_value();
    }
    for i in 0..8 {
      self.priority[i] = u32::max_value();
    }
  }

  fn set_enable(&mut self, int: usize) {
    let enable_index = int / 32;
    let bit = 1u32 << (int % 32);
    self.set_enable[enable_index] = bit;
  }

  fn clear_enable(&mut self, int: usize) {
    let enable_index = int / 32;
    let bit = 1u32 << (int % 32);
    self.clear_enable[enable_index] = bit;
  }

  fn set_target(&mut self, int: usize, target: u8) {
    let target_index = (int * 8) / 32;
    let offset = (int * 8) % 32;
    let mask: u32 = 0b11111111 << offset;
    self.target[target_index] =
        (self.target[target_index] & (!mask)) | (((target as u32) << offset) & mask);
  }

  fn set_priority(&mut self, int: usize, priority: u8) {
    let priority_index = (int * 8) / 32;
    let offset = (int * 8) % 32;
    let mask: u32 = 0b11111111 << offset;
    self.priority[priority_index] =
        (self.priority[priority_index] & (!mask)) | (((priority as u32) << offset) & mask);
  }

}

pub fn init() {
  let gicd = unsafe { (GICD_BASE.pa2kva() as *mut GicDistributor).as_mut().unwrap() };
  gicd.init();
}

pub fn init_per_core() {
  let gicd = unsafe { (GICD_BASE.pa2kva() as *mut GicDistributor).as_mut().unwrap() };
  let gicc = unsafe { (GICC_BASE.pa2kva() as *mut GicCpuInterface).as_mut().unwrap() };
  gicd.init_per_core();
  gicc.init();
}

pub fn enable_interrupt(int: usize, target: u8) {
  let gicd = unsafe { (GICD_BASE.pa2kva() as *mut GicDistributor).as_mut().unwrap() };
  gicd.set_enable(int);
  gicd.set_priority(int, 0x7f);
  gicd.set_target(int, target);
}

pub fn clear_interrupt(int: usize) {
  let gicc = unsafe { (GICC_BASE.pa2kva() as *mut GicCpuInterface).as_mut().unwrap() };
  gicc.end_of_int = int as u32;
}