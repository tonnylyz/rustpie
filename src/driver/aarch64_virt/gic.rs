use crate::arch::{Address, ArchTrait};
use crate::lib::interrupt::{InterruptController, InterruptNo};

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
    self.pmr = u32::MAX;
    self.control |= 1 | (1 << 9);
  }
}

impl GicDistributor {
  fn init(&mut self) {
    let max_spi = (self.typer & 0b11111) * 32 + 1;
    for i in 1usize..(max_spi as usize / 32) {
      self.clear_enable[i] = u32::MAX;
      self.clear_pend[i] = u32::MAX;
      self.clear_active[i] = u32::MAX;
    }
    for i in 8usize..(max_spi as usize * 8 / 32) {
      self.priority[i] = u32::MAX;
      self.target[i] = u32::MAX;
    }
    self.control |= 1;
  }

  fn init_per_core(&mut self) {
    self.clear_enable[0] = u32::MAX;
    self.clear_pend[0] = u32::MAX;
    self.clear_active[0] = u32::MAX;
    for i in 0..4 {
      self.clear_pend_sgi[i] = u32::MAX;
    }
    for i in 0..8 {
      self.priority[i] = u32::MAX;
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

fn gicd() -> &'static mut GicDistributor {
  unsafe { (GICD_BASE.pa2kva() as *mut GicDistributor).as_mut().unwrap() }
}

fn gicc() -> &'static mut GicCpuInterface {
  unsafe { (GICC_BASE.pa2kva() as *mut GicCpuInterface).as_mut().unwrap() }
}

pub struct Gic;

impl InterruptController for Gic {
  fn init(&self) {
    let core_id = crate::arch::Arch::core_id();
    let gicd = gicd();
    if core_id == 0 {
      gicd.init();
    }
    let gicc = gicc();
    gicd.init_per_core();
    gicc.init();
  }

  fn enable(&self, int: InterruptNo) {
    match int {
      InterruptNo::Timer => {panic!("GIC use numbered timer irq");}
      InterruptNo::Numbered(int) => {
        let core_id = crate::arch::Arch::core_id();
        let gicd = gicd();
        gicd.set_enable(int);
        gicd.set_priority(int, 0x7f);
        gicd.set_target(int, (1 << core_id) as u8);
      }
    }
  }

  fn disable(&self, int: InterruptNo) {
    match int {
      InterruptNo::Timer => {panic!("GIC use numbered timer irq");}
      InterruptNo::Numbered(int) => {
        let gicd = gicd();
        gicd.clear_enable(int);
      }
    }
  }

  fn fetch(&self) -> Option<InterruptNo> {
    let gicc = gicc();
    let i = gicc.int_ack;
    if i >= 1022 {
      None
    } else {
      Some(InterruptNo::Numbered(i as usize))
    }
  }

  fn finish(&self, int: InterruptNo) {
    match int {
      InterruptNo::Timer => {panic!("GIC use numbered timer irq");}
      InterruptNo::Numbered(int) => {
        let gicc = gicc();
        gicc.end_of_int = int as u32;
      }
    }
  }
}

pub const INT_TIMER: InterruptNo = InterruptNo::Numbered(27);

pub static INTERRUPT_CONTROLLER: Gic = Gic{};
