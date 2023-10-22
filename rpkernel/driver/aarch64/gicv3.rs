use core::arch::asm;

use spin::Once;
use tock_registers::interfaces::{Readable, Writeable};
use tock_registers::registers::*;
use tock_registers::*;

use crate::core_id;
use crate::kernel::interrupt::{InterProcessorInterruptController, InterruptController};

const GIC_INTERRUPT_NUM: usize = 1024;
const GIC_SGI_NUM: usize = 16;
const GIC_PPI_NUM: usize = 16;
const GIC_1_BIT_NUM: usize = GIC_INTERRUPT_NUM / 32;
const GIC_8_BIT_NUM: usize = GIC_INTERRUPT_NUM * 8 / 32;
const GIC_2_BIT_NUM: usize = GIC_INTERRUPT_NUM * 2 / 32;

register_bitfields! {
  u32,
  pub GICD_TYPE [
    ITLinesNumber OFFSET(0) NUMBITS(5) [],
  ],
}

register_structs! {
  #[allow(non_snake_case)]
  GicDistributorMmio {
    (0x0000 => Control: ReadWrite<u32>),
    (0x0004 => Type: ReadOnly<u32, GICD_TYPE::Register>),
    (0x0008 => ImplementerId: ReadOnly<u32>),
    (0x000c => Type2: ReadOnly<u32>),
    (0x0010 => Status: ReadWrite<u32>),
    (0x0014 => _reserved_0),
    (0x0040 => SetSpiNs: WriteOnly<u32>),
    (0x0044 => _reserved_1),
    (0x0048 => ClearSpiNs: WriteOnly<u32>),
    (0x004c => _reserved_2),
    (0x0050 => SetSpiS: WriteOnly<u32>),
    (0x0054 => _reserved_3),
    (0x0058 => ClearSpiS: WriteOnly<u32>),
    (0x005c => _reserved_4),
    (0x0080 => IntGroup: [ReadWrite<u32>; GIC_1_BIT_NUM]),
    (0x0100 => IntSetEnable: [ReadWrite<u32>; GIC_1_BIT_NUM]),
    (0x0180 => IntClearEnable: [ReadWrite<u32>; GIC_1_BIT_NUM]),
    (0x0200 => IntSetPending: [ReadWrite<u32>; GIC_1_BIT_NUM]),
    (0x0280 => IntClearPending: [ReadWrite<u32>; GIC_1_BIT_NUM]),
    (0x0300 => IntSetActive: [ReadWrite<u32>; GIC_1_BIT_NUM]),
    (0x0380 => IntClearActive: [ReadWrite<u32>; GIC_1_BIT_NUM]),
    (0x0400 => IntPriority: [ReadWrite<u32>; GIC_8_BIT_NUM]),
    // (0x0800 => IntTargets: [ReadWrite<u32>; GIC_8_BIT_NUM]),
    (0x0800 => _LEGACY_IntTargets),
    (0x0c00 => IntConfig: [ReadWrite<u32>; GIC_2_BIT_NUM]),
    (0x0d00 => IntGroupMod: [ReadWrite<u32>; GIC_2_BIT_NUM]),
    (0x0e00 => NsAccessControl: [ReadWrite<u32>; GIC_2_BIT_NUM]),
    (0x0f00 => _LEGACY_SGIR),
    // Extended SPI range
    (0x6100 => IntRoute: [ReadWrite<u64>; GIC_INTERRUPT_NUM - GIC_SGI_NUM - GIC_PPI_NUM]),
    (0x8000 => _extended_route),
    (0x10000 => @END),
  },

  #[allow(non_snake_case)]
  GicRedistributorRdMmio {
    (0x0000 => Control: ReadWrite<u32>),
    (0x0004 => ImplementerId: ReadOnly<u32>),
    (0x0008 => Type: ReadOnly<u64>),
    (0x0010 => Status: ReadWrite<u32>),
    (0x0014 => Wake: ReadWrite<u32>),
    (0x0018 => MaxPartIdPMG: ReadOnly<u32>),
    (0x001C => PartIdPMG: ReadWrite<u32>),
    (0x0020 => _reserved_0),
    (0x0040 => SetLpiPending: WriteOnly<u64>),
    (0x0048 => ClearLpiPending: WriteOnly<u64>),
    (0x0050 => _reserved_1),
    (0x0070 => PropertiesBaseAddr: ReadWrite<u64>),
    (0x0078 => LpiPendingTableBaseAddr: ReadWrite<u64>),
    (0x0080 => _LPI),
    (0x10000 => @END),
  },

  #[allow(non_snake_case)]
  GicRedistributorSgiMmio {
    (0x0000 => _reserved_0),
    (0x0080 => IntGroup0: ReadWrite<u32>),
    (0x0084 => _igroupr_n_ext),
    (0x0100 => IntSetEnable0: ReadWrite<u32>),
    (0x0104 => _isenabler_n_ext),
    (0x0180 => IntClearEnable0: ReadWrite<u32>),
    (0x0184 => _icenabler_n_ext),
    (0x0200 => IntSetPending0: ReadWrite<u32>),
    (0x0204 => _ispendr_n_ext),
    (0x0280 => IntClearPending0: ReadWrite<u32>),
    (0x0284 => _icpendr_n_ext),
    (0x0300 => IntSetActive0: ReadWrite<u32>),
    (0x0304 => _isactiver_n_ext),
    (0x0380 => IntClearActive0: ReadWrite<u32>),
    (0x0384 => _icactiver_n_ext),
    (0x0400 => IntPriority: [ReadWrite<u32>; 8]),
    (0x0420 => _ipriorityr_n_ext),
    (0x0c00 => IntConfig0: ReadWrite<u32>),
    (0x0c04 => IntConfig1: ReadWrite<u32>),
    (0x0c08 => _icfgr_n_ext),
    (0x0d00 => IntGroupMod0: ReadWrite<u32>),
    (0x0d04 => _igrpmodr_n_ext),
    (0x0e00 => NsAccessControl: ReadWrite<u32>),
    (0x0e04 => _nsacr),
    (0x10000 => @END),
  }
}

struct GicDistributor {
  base_addr: usize,
}

impl core::ops::Deref for GicDistributor {
  type Target = GicDistributorMmio;

  fn deref(&self) -> &Self::Target {
    unsafe { &*self.ptr() }
  }
}

impl GicDistributor {
  fn ptr(&self) -> *const GicDistributorMmio {
    self.base_addr as *const _
  }

  fn init(&self) {
    debug!("gicd_type {:x}", self.Type.get());
    debug!("gicd_type2 {:x}", self.Type2.get());
    self.Control.set(0); // disable the distributor
    self.wait_for_rwp();

    let itlines_num = self.Type.read(GICD_TYPE::ITLinesNumber) as usize;
    let line_num = (itlines_num + 1) * 32;
    for i in 1..(itlines_num + 1) {
      self.IntGroup[i].set(u32::MAX);
      self.IntClearEnable[i].set(u32::MAX);
      self.IntClearPending[i].set(u32::MAX);
      self.IntClearActive[i].set(u32::MAX);
    }
    for i in 8..8 * (itlines_num + 1) {
      self.IntPriority[i].set(0xa0_a0_a0_a0);
    }
    for i in 2..2 * (itlines_num + 1) {
      self.IntConfig[i].set(0xaaaaaaaa); // All set to edge
    }
    self.Control.set(
      (1 << 4/* ARE_NS */) | (1 << 1/* EnableGrp1NS */) | 1, /* EnableGrp1NS */
    );
    self.wait_for_rwp();
    debug!("gicd_ctrl {:b}", self.Control.get());

    let def_affinity = 0; // MPIDR -> aff
    for i in 32..line_num {
      self.IntRoute[i].set(def_affinity);
    }
  }

  fn set_enable(&self, int: usize) {
    assert!(int < 1024);
    let idx = int / 32;
    let bit = 1u32 << (int % 32);
    self.IntSetEnable[idx].set(bit);
  }

  fn clear_enable(&self, int: usize) {
    assert!(int < 1024);
    let idx = int / 32;
    let bit = 1u32 << (int % 32);
    self.IntClearEnable[idx].set(bit);
    self.wait_for_rwp();
  }

  fn wait_for_rwp(&self) {
    loop {
      if (self.Control.get() & (1 << 3/* RWP */)) == 0 {
        break;
      }
    }
  }
}

struct GicRedistributor {
  rd_base: usize,
  sgi_base: usize,
}

impl GicRedistributor {
  fn rd_ptr(&self) -> *const GicRedistributorRdMmio {
    self.rd_base as *const _
  }

  fn sgi_ptr(&self) -> *const GicRedistributorSgiMmio {
    self.sgi_base as *const _
  }

  fn rd(&self) -> &GicRedistributorRdMmio {
    unsafe { &*self.rd_ptr() }
  }

  fn sgi(&self) -> &GicRedistributorSgiMmio {
    unsafe { &*self.sgi_ptr() }
  }

  fn init(&self) {
    let typer = self.rd().Type.get();
    let affinity_value = (typer >> 32) as u32;
    debug!("aff {}", affinity_value);

    let mut wake = self.rd().Wake.get();
    wake &= !(1u32 << 1); // clear ProcessorSleep
    self.rd().Wake.set(wake);
    loop {
      if self.rd().Wake.get() & (1u32 << 2)/* ChildrenAsleep */ == 0 {
        break;
      }
    }

    self.sgi().IntGroup0.set(u32::MAX); // set SGIs and PPIs to be NS Group 1

    // self.sgi().IntClearPending0.set(u32::MAX);
    self.sgi().IntClearActive0.set(u32::MAX);
    self.sgi().IntClearEnable0.set(u32::MAX);
    for i in 0..8 {
      self.sgi().IntPriority[i].set(0xa0_a0_a0_a0);
    }
    self.wait_for_rwp();
    // info!("cfg 0 {:x} cfg 1 {:x}", self.sgi().IntConfig0.get(), self.sgi().IntConfig1.get());
  }

  fn set_enable(&self, int: usize) {
    assert!(int < 32);
    let bit = 1u32 << int;
    self.sgi().IntSetEnable0.set(bit);
    // info!("gicr int set enable 0 {:x}", self.sgi().IntSetEnable0.get());
  }

  fn clear_enable(&self, int: usize) {
    assert!(int < 32);
    let bit = 1u32 << int;
    self.sgi().IntClearEnable0.set(bit);
    self.wait_for_rwp();
  }

  // fn set_priority(&self, int: usize, priority: u8) {
  //   assert!(int < 32);
  //   let reg_idx = int / 8;
  //   let offset = (int % 4) * 8;
  //   let mask: u32 = 0xff << offset;
  //   let prev = self.sgi().IntPriority[reg_idx].get();
  //   self.sgi().IntPriority[reg_idx].set((prev & (!mask)) | (((priority as u32) << offset) & mask))
  // }

  fn wait_for_rwp(&self) {
    loop {
      if (self.rd().Control.get() & (1 << 3/* RWP */)) == 0 {
        break;
      }
    }
  }
}

struct GicCpuInterface {}

impl GicCpuInterface {
  fn int_ack(&self) -> u32 {
    let r: u32;
    unsafe {
      asm!("mrs {:x}, ICC_IAR1_EL1", out(reg) r);
    }
    r
  }

  fn write_end_of_int(&self, value: u32) {
    unsafe {
      asm!("msr ICC_EOIR1_EL1, {:x}", in(reg) value);
    }
  }

  fn write_soft_gen_int_1(&self, value: u64) {
    unsafe {
      asm!("msr ICC_SGI1R_EL1, {}", in(reg) value);
    }
  }

  fn write_priority_mask(&self, value: u32) {
    unsafe {
      asm!("msr ICC_PMR_EL1, {:x}", in(reg) value);
    }
  }

  fn control(&self) -> u32 {
    let r: u32;
    unsafe {
      asm!("mrs {:x}, ICC_CTLR_EL1", out(reg) r);
    }
    r
  }

  fn write_control(&self, value: u32) {
    unsafe {
      asm!("msr ICC_CTLR_EL1, {:x}", in(reg) value);
    }
  }

  fn write_bpr1(&self, value: u64) {
    unsafe {
      asm!("msr ICC_BPR1_EL1, {}", in(reg) value);
    }
  }

  fn write_sre(&self, value: u64) {
    unsafe {
      asm!("msr ICC_SRE_EL1, {}", in(reg) value);
    }
  }

  fn write_group1_enable(&self, value: u64) {
    unsafe {
      asm!("msr ICC_IGRPEN1_EL1, {}", in(reg) value);
    }
  }

  fn init(&self) {
    self.write_sre(1 /* System Register Enable */);
    let priority_bits = (self.control() >> 8) & 0b111;
    debug!("gicc ctrl pribits {}", priority_bits);
    self.write_priority_mask(0xf0);
    self.write_bpr1(0);
    self.write_control(0); // EOImode0
                           // TODO: program ICC_AP1R0_EL1 based on priority_bits
    self.write_group1_enable(1);
  }
}

pub struct Gic {
  d: GicDistributor,
  r: [Option<GicRedistributor>; crate::MAX_CPU_NUMBER],
  c: GicCpuInterface,
}

const NONE_GICR: Option<GicRedistributor> = None;

impl Gic {
  fn new() -> Self {
    let mut r = Gic {
      d: GicDistributor {
        base_addr: 0x8000000usize.pa2kva(),
      },
      r: [NONE_GICR; crate::MAX_CPU_NUMBER],
      c: GicCpuInterface {},
    };
    let gicr_base = 0x80a0000usize.pa2kva();
    for i in 0..crate::cpu_number() {
      r.r[i] = Some(GicRedistributor {
        rd_base: gicr_base + i * 0x20000,
        sgi_base: gicr_base + i * 0x20000 + 0x10000,
      })
    }
    r
  }
}

impl InterruptController for Once<Gic> {
  fn init(&self) {
    let gic = self.call_once(|| Gic::new());
    let core_id = core_id();
    if core_id == 0 {
      gic.d.init();
    }
    crate::util::barrier();
    gic.r[core_id].as_ref().unwrap().init();
    gic.c.init();
  }

  fn enable(&self, int: super::Interrupt) {
    let gic = self.get().unwrap();
    let core_id = core_id();
    let gicr = gic.r[core_id].as_ref().unwrap();
    let gicd = &gic.d;
    match int {
      0..=15 => {
        // SGI
        gicr.set_enable(int);
      }
      16..=31 => {
        // PPI
        gicr.set_enable(int);
      }
      _ => {
        // SPI
        gicd.set_enable(int);
      }
    }
  }

  fn disable(&self, int: super::Interrupt) {
    let gic = self.get().unwrap();
    let core_id = core_id();
    let gicr = gic.r[core_id].as_ref().unwrap();
    let gicd = &gic.d;
    match int {
      0..=15 => {
        // SGI
        gicr.clear_enable(int);
      }
      16..=31 => {
        // PPI
        gicr.clear_enable(int);
      }
      _ => {
        // SPI
        gicd.clear_enable(int);
      }
    }
  }

  fn fetch(&self) -> Option<(super::Interrupt, usize)> {
    let int_id = self.get().unwrap().c.int_ack();
    if int_id >= 1020 {
      None
    } else {
      Some((int_id as Interrupt, 0 as usize)) // TODO: source!
    }
  }

  fn finish(&self, int: super::Interrupt) {
    self.get().unwrap().c.write_end_of_int(int as u32);
  }
}

pub const INT_TIMER: Interrupt = 27; // virtual timer

pub static INTERRUPT_CONTROLLER: Once<Gic> = Once::new();

pub type Interrupt = usize;

use crate::kernel::interrupt::InterProcessInterrupt as IPI;
use crate::kernel::traits::Address;

impl InterProcessorInterruptController for Once<Gic> {
  fn send_to_one(&self, irq: IPI, target: usize) {
    assert!(target != core_id());
    self.send_to_multiple(irq, 1usize << target);
    // TODO: the mask only applies to cores in the same cluster
  }

  fn send_to_multiple(&self, irq: IPI, target_mask: usize) {
    let int_id: Interrupt = irq.into();
    assert!(int_id < 16);
    assert!(target_mask != 0);
    assert!(target_mask < 0x10000);
    self
      .get()
      .unwrap()
      .c
      .write_soft_gen_int_1((int_id << 24) as u64 | (target_mask as u64));
  }
}

// should move to arch
impl From<IPI> for Interrupt {
  fn from(value: IPI) -> Self {
    match value {
      IPI::IPI0 => 0,
    }
  }
}

impl From<Interrupt> for IPI {
  fn from(value: Interrupt) -> Self {
    match value {
      0 => IPI::IPI0,
      _ => panic!(),
    }
  }
}
