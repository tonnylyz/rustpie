use tock_registers::interfaces::{Readable, Writeable};
use tock_registers::registers::*;
use tock_registers::*;

use crate::kernel::interrupt::{InterProcessorInterruptController, InterruptController};
use crate::kernel::traits::ArchTrait;

const GIC_INTERRUPT_NUM: usize = 1024;
const GIC_SGI_NUM: usize = 16;
const GIC_1_BIT_NUM: usize = GIC_INTERRUPT_NUM / 32;
const GIC_8_BIT_NUM: usize = GIC_INTERRUPT_NUM * 8 / 32;
const GIC_2_BIT_NUM: usize = GIC_INTERRUPT_NUM * 2 / 32;

#[cfg(not(feature = "tx2"))]
const GICD_BASE: usize = 0x08000000;
#[cfg(not(feature = "tx2"))]
const GICC_BASE: usize = 0x08010000;


#[cfg(feature = "tx2")]
const GICD_BASE: usize = 0x3881000;
#[cfg(feature = "tx2")]
const GICC_BASE: usize = 0x3882000;

register_bitfields! {
  u32,
  pub GICD_SGIR [
    TargetListFilter OFFSET(24) NUMBITS(2) [
      ForwardInList        = 0b00,
      ForwardAllExceptSelf = 0b01,
      ForwardSelf          = 0b10,
    ],
    CpuTargetList OFFSET(16) NUMBITS(8) [],
    NSATT OFFSET(15) NUMBITS(1) [],
    SGIIntId OFFSET(0) NUMBITS(4) [],
  ],
  pub GICC_IAR [
    CPUID OFFSET(10) NUMBITS(3) [],
    INTID OFFSET(0) NUMBITS(10) [],
  ],
}

register_structs! {
  #[allow(non_snake_case)]
  GicDistributorBlock {
    (0x0000 => CTLR: ReadWrite<u32>),
    (0x0004 => TYPER: ReadOnly<u32>),
    (0x0008 => IIDR: ReadOnly<u32>),
    (0x000c => _reserved_0),
    (0x0080 => IGROUPR: [ReadWrite<u32>; GIC_1_BIT_NUM]),
    (0x0100 => ISENABLER: [ReadWrite<u32>; GIC_1_BIT_NUM]),
    (0x0180 => ICENABLER: [ReadWrite<u32>; GIC_1_BIT_NUM]),
    (0x0200 => ISPENDR: [ReadWrite<u32>; GIC_1_BIT_NUM]),
    (0x0280 => ICPENDR: [ReadWrite<u32>; GIC_1_BIT_NUM]),
    (0x0300 => ISACTIVER: [ReadWrite<u32>; GIC_1_BIT_NUM]),
    (0x0380 => ICACTIVER: [ReadWrite<u32>; GIC_1_BIT_NUM]),
    (0x0400 => IPRIORITYR: [ReadWrite<u32>; GIC_8_BIT_NUM]),
    (0x0800 => ITARGETSR: [ReadWrite<u32>; GIC_8_BIT_NUM]),
    (0x0c00 => ICFGR: [ReadWrite<u32>; GIC_2_BIT_NUM]),
    (0x0d00 => _reserved_1),
    (0x0e00 => NSACR: [ReadWrite<u32>; GIC_2_BIT_NUM]),
    (0x0f00 => SGIR: WriteOnly<u32, GICD_SGIR::Register>),
    (0x0f04 => _reserved_2),
    (0x0f10 => CPENDSGIR: [ReadWrite<u32>; GIC_SGI_NUM * 8 / 32]),
    (0x0f20 => SPENDSGIR: [ReadWrite<u32>; GIC_SGI_NUM * 8 / 32]),
    (0x0f30 => _reserved_3),
    (0x1000 => @END),
  }
}

struct GicDistributor {
  base_addr: usize,
  cpu_if_id: spin::Mutex<[u8; crate::board::BOARD_CORE_NUMBER]>,
}

impl core::ops::Deref for GicDistributor {
  type Target = GicDistributorBlock;

  fn deref(&self) -> &Self::Target {
    unsafe { &*self.ptr() }
  }
}

register_structs! {
  #[allow(non_snake_case)]
  GicCpuInterfaceBlock {
    (0x0000 => CTLR: ReadWrite<u32>),   // CPU Interface Control Register
    (0x0004 => PMR: ReadWrite<u32>),    // Interrupt Priority Mask Register
    (0x0008 => BPR: ReadWrite<u32>),    // Binary Point Register
    (0x000c => IAR: ReadOnly<u32, GICC_IAR::Register>),     // Interrupt Acknowledge Register
    (0x0010 => EOIR: WriteOnly<u32>),   // End of Interrupt Register
    (0x0014 => RPR: ReadOnly<u32>),     // Running Priority Register
    (0x0018 => HPPIR: ReadOnly<u32>),   // Highest Priority Pending Interrupt Register
    (0x001c => ABPR: ReadWrite<u32>),   // Aliased Binary Point Register
    (0x0020 => AIAR: ReadOnly<u32>),    // Aliased Interrupt Acknowledge Register
    (0x0024 => AEOIR: WriteOnly<u32>),  // Aliased End of Interrupt Register
    (0x0028 => AHPPIR: ReadOnly<u32>),  // Aliased Highest Priority Pending Interrupt Register
    (0x002c => _reserved_0),
    (0x00d0 => APR: [ReadWrite<u32>; 4]),    // Active Priorities Register
    (0x00e0 => NSAPR: [ReadWrite<u32>; 4]),  // Non-secure Active Priorities Register
    (0x00f0 => _reserved_1),
    (0x00fc => IIDR: ReadOnly<u32>),    // CPU Interface Identification Register
    (0x0100 => _reserved_2),
    (0x1000 => DIR: WriteOnly<u32>),    // Deactivate Interrupt Register
    (0x1004 => _reserved_3),
    (0x2000 => @END),
  }
}

struct GicCpuInterface {
  base_addr: usize,
}

impl core::ops::Deref for GicCpuInterface {
  type Target = GicCpuInterfaceBlock;

  fn deref(&self) -> &Self::Target {
    unsafe { &*self.ptr() }
  }
}

impl GicCpuInterface {
  const fn new(base_addr: usize) -> Self {
    GicCpuInterface { base_addr }
  }

  fn ptr(&self) -> *const GicCpuInterfaceBlock {
    self.base_addr as *const _
  }

  fn init(&self) {
    self.PMR.set(u32::MAX);
    self.CTLR.set(1);
  }
}

impl GicDistributor {
  const fn new(base_addr: usize) -> Self {
    GicDistributor { base_addr, cpu_if_id: spin::Mutex::new([0; crate::board::BOARD_CORE_NUMBER]) }
  }

  fn ptr(&self) -> *const GicDistributorBlock {
    self.base_addr as *const _
  }

  fn init(&self) {
    let max_spi = (self.TYPER.get() & 0b11111) * 32 + 1;
    for i in 1usize..(max_spi as usize / 32) {
      self.ICENABLER[i].set(u32::MAX);
      self.ICPENDR[i].set(u32::MAX);
      self.ICACTIVER[i].set(u32::MAX);
    }
    for i in 8usize..(max_spi as usize * 8 / 32) {
      self.IPRIORITYR[i].set(u32::MAX);
      self.ITARGETSR[i].set(u32::MAX);
    }
    self.CTLR.set(1);
  }

  fn init_per_core(&self) {
    self.ICENABLER[0].set(u32::MAX);
    self.ICPENDR[0].set(u32::MAX);
    self.ICACTIVER[0].set(u32::MAX);
    for i in 0..4 {
      self.CPENDSGIR[i].set(u32::MAX);
    }
    for i in 0..8 {
      self.IPRIORITYR[i].set(u32::MAX);
    }
    let cpu_if_id = (self.ITARGETSR[0].get() & 0xff) as u8;
    info!("cpu_if_id {}", cpu_if_id);
    let mut ids = self.cpu_if_id.lock();
    ids[crate::arch::Arch::core_id()] = cpu_if_id;
  }

  fn set_enable(&self, int: usize) {
    // SGIs may be always enabled and thus don't need enabling
    let idx = int / 32;
    let bit = 1u32 << (int % 32);
    self.ISENABLER[idx].set(bit);
  }

  fn clear_enable(&self, int: usize) {
    let idx = int / 32;
    let bit = 1u32 << (int % 32);
    self.ICENABLER[idx].set(bit);
  }

  fn set_target(&self, int: usize, target: u8) {
    if int < 32 {
      // GICD_ITARGETSR0 to GICD_ITARGETSR7 are read-only
      return;
    }
    let idx = (int * 8) / 32;
    let offset = (int * 8) % 32;
    let mask: u32 = 0b11111111 << offset;
    let prev = self.ITARGETSR[idx].get();
    self.ITARGETSR[idx].set((prev & (!mask)) | (((target as u32) << offset) & mask));
  }

  fn set_priority(&self, int: usize, priority: u8) {
    let idx = (int * 8) / 32;
    let offset = (int * 8) % 32;
    let mask: u32 = 0b11111111 << offset;
    let prev = self.IPRIORITYR[idx].get();
    self.IPRIORITYR[idx].set((prev & (!mask)) | (((priority as u32) << offset) & mask));
  }

  fn set_config(&self, int: usize, edge: bool) {
    let idx = (int * 2) / 32;
    let offset = (int * 2) % 32;
    let mask: u32 = 0b11 << offset;
    let prev = self.ICFGR[idx].get();
    self.ICFGR[idx].set((prev & (!mask)) | ((if edge { 0b10 } else { 0b00 } << offset) & mask));
  }
}

static GICD: GicDistributor = GicDistributor::new(GICD_BASE | 0xFFFF_FF80_0000_0000);
static GICC: GicCpuInterface = GicCpuInterface::new(GICC_BASE | 0xFFFF_FF80_0000_0000);

pub struct Gic;

impl InterruptController for Gic {
  fn init(&self) {
    let core_id = crate::arch::Arch::core_id();
    let gicd = &GICD;
    if core_id == 0 {
      gicd.init();
    }
    crate::util::barrier();
    let gicc = &GICC;
    gicd.init_per_core();
    gicc.init();
  }

  fn enable(&self, int: Interrupt) {
    let core_id = crate::arch::Arch::core_id();
    let gicd = &GICD;
    gicd.set_enable(int);
    gicd.set_priority(int, 0x7f);
    if int < 16 {
      return;
    }
    if int >= 32 {
      // when enabling SPIs, set it to be edge triggered and targeting current CPU
      gicd.set_config(int, true);
      gicd.set_target(int, (1 << core_id) as u8);
    }
  }

  fn disable(&self, int: Interrupt) {
    let gicd = &GICD;
    gicd.clear_enable(int);
  }

  fn fetch(&self) -> Option<(Interrupt, usize)> {
    let gicc = &GICC;
    let iar = gicc.IAR.extract();
    let int_id = iar.read(GICC_IAR::INTID);
    let src_cpu_id = iar.read(GICC_IAR::CPUID);
    if int_id >= 1022 {
      None
    } else {
      Some((int_id as Interrupt, src_cpu_id as usize))
    }
  }

  fn finish(&self, int: Interrupt) {
    let gicc = &GICC;
    gicc.EOIR.set(int as u32);
  }
}

pub const INT_TIMER: Interrupt = 27; // virtual timer

pub static INTERRUPT_CONTROLLER: Gic = Gic {};

pub type Interrupt = usize;

use crate::kernel::interrupt::InterProcessInterrupt as IPI;

impl InterProcessorInterruptController for Gic {
  fn send_to_one(&self, irq: IPI, target: usize) {
    assert!(target != crate::arch::Arch::core_id());
    self.send_to_multiple(irq, 1usize << target);
  }

  fn send_to_multiple(&self, irq: IPI, target_mask: usize) {
    let gicd = &GICD;
    let sgi_int_id: Interrupt = irq.into();
    assert!(sgi_int_id < 16); // SGI INT id range is 0-15
    assert!(target_mask != 0);
    assert!(target_mask <= 0b1111_1111); // GICv2 supports at most 8 CPU interfaces
    let mut if_mask = 0u8;
    let if_ids = gicd.cpu_if_id.lock();
    for i in 0..8 {
      if target_mask & (1 << i) != 0 {
        assert!(if_ids[i] != 0);
        if_mask |= if_ids[i];
      }
    }
    gicd.SGIR.write(
      GICD_SGIR::CpuTargetList.val(if_mask as u8 as u32) + GICD_SGIR::SGIIntId.val(sgi_int_id as u32),
    );
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
