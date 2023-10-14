use spin::Mutex;
use tock_registers::registers::*;
use tock_registers::{
  interfaces::{Readable, Writeable},
  register_structs,
};

use crate::core_id;
use crate::kernel::interrupt::{
  InterProcessInterrupt as IPI, InterProcessorInterruptController, InterruptController,
};

// platform level interrupt controller
// https://github.com/riscv/riscv-plic-spec/blob/master/riscv-plic.adoc
const PLIC_BASE_ADDR: usize = 0xffff_ffff_0000_0000 + 0x0c00_0000;

pub struct Rv64InterruptController {
  ipi_mailboxes: Mutex<[Option<(IPI, usize)>; crate::MAX_CPU_NUMBER]>,
}

register_structs! {
  #[allow(non_snake_case)]
  PlicBlock {
    (0x000000 => InterruptPriority: [ReadWrite<u32>; 0x400]),
    (0x001000 => InterruptPending: [ReadOnly<u32>; 0x20]),
    (0x001080 => _reserved_0),
    (0x002000 => InterruptEnableCtx0: [ReadWrite<u32>; 0x20]),
    (0x002080 => InterruptEnableCtx1: [ReadWrite<u32>; 0x20]),
    (0x002100 => InterruptEnableCtx2: [ReadWrite<u32>; 0x20]),
    (0x002180 => InterruptEnableCtx3: [ReadWrite<u32>; 0x20]),
    (0x002200 => InterruptEnableCtx4: [ReadWrite<u32>; 0x20]),
    (0x002280 => InterruptEnableCtx5: [ReadWrite<u32>; 0x20]),
    (0x002300 => InterruptEnableCtx6: [ReadWrite<u32>; 0x20]),
    (0x002380 => InterruptEnableCtx7: [ReadWrite<u32>; 0x20]),
    (0x002400 => _reserved_1),
    (0x200000 => PriorityThresholdCtx0: ReadWrite<u32>),
    (0x200004 => InterruptClaimCompletionCtx0: ReadWrite<u32>),
    (0x200008 => _reserved_2),
    (0x201000 => PriorityThresholdCtx1: ReadWrite<u32>),
    (0x201004 => InterruptClaimCompletionCtx1: ReadWrite<u32>),
    (0x201008 => _reserved_3),
    (0x202000 => PriorityThresholdCtx2: ReadWrite<u32>),
    (0x202004 => InterruptClaimCompletionCtx2: ReadWrite<u32>),
    (0x202008 => _reserved_4),
    (0x203000 => PriorityThresholdCtx3: ReadWrite<u32>),
    (0x203004 => InterruptClaimCompletionCtx3: ReadWrite<u32>),
    (0x203008 => _reserved_5),
    (0x204000 => PriorityThresholdCtx4: ReadWrite<u32>),
    (0x204004 => InterruptClaimCompletionCtx4: ReadWrite<u32>),
    (0x204008 => _reserved_6),
    (0x205000 => PriorityThresholdCtx5: ReadWrite<u32>),
    (0x205004 => InterruptClaimCompletionCtx5: ReadWrite<u32>),
    (0x205008 => _reserved_7),
    (0x206000 => PriorityThresholdCtx6: ReadWrite<u32>),
    (0x206004 => InterruptClaimCompletionCtx6: ReadWrite<u32>),
    (0x206008 => _reserved_8),
    (0x207000 => PriorityThresholdCtx7: ReadWrite<u32>),
    (0x207004 => InterruptClaimCompletionCtx7: ReadWrite<u32>),
    (0x207008 => _reserved_9),
    (0x210000 => @END),
  }
}

struct PlicMmio {
  base_addr: usize,
}

impl core::ops::Deref for PlicMmio {
  type Target = PlicBlock;
  fn deref(&self) -> &Self::Target {
    unsafe { &*self.ptr() }
  }
}

impl PlicMmio {
  const fn new(base_addr: usize) -> Self {
    PlicMmio { base_addr }
  }
  fn ptr(&self) -> *const PlicBlock {
    self.base_addr as *const _
  }
}

static PLIC_MMIO: PlicMmio = PlicMmio::new(PLIC_BASE_ADDR);

impl InterruptController for Rv64InterruptController {
  fn init(&self) {
    let plic = &PLIC_MMIO;
    let core_id = core_id();
    match core_id {
      0 => plic.PriorityThresholdCtx1.set(0),
      1 => plic.PriorityThresholdCtx3.set(0),
      2 => plic.PriorityThresholdCtx5.set(0),
      3 => plic.PriorityThresholdCtx7.set(0),
      _ => panic!(),
    }
  }

  fn enable(&self, i: Interrupt) {
    let plic = &PLIC_MMIO;
    let core_id = core_id();
    let reg_idx = i / 32;
    let bit_idx = i % 32;
    let bit_mask: u32 = 1 << bit_idx;

    match core_id {
      0 => {
        plic.InterruptEnableCtx1[reg_idx].set(plic.InterruptEnableCtx1[reg_idx].get() | bit_mask)
      }
      1 => {
        plic.InterruptEnableCtx3[reg_idx].set(plic.InterruptEnableCtx3[reg_idx].get() | bit_mask)
      }
      2 => {
        plic.InterruptEnableCtx5[reg_idx].set(plic.InterruptEnableCtx5[reg_idx].get() | bit_mask)
      }
      3 => {
        plic.InterruptEnableCtx7[reg_idx].set(plic.InterruptEnableCtx7[reg_idx].get() | bit_mask)
      }
      _ => panic!(),
    }
    plic.InterruptPriority[i].set(1);
  }

  fn disable(&self, i: Interrupt) {
    let plic = &PLIC_MMIO;
    let core_id = core_id();
    let reg_idx = i / 32;
    let bit_idx = i % 32;
    let bit_mask: u32 = 1 << bit_idx;
    match core_id {
      0 => {
        plic.InterruptEnableCtx1[reg_idx].set(plic.InterruptEnableCtx1[reg_idx].get() & !bit_mask)
      }
      1 => {
        plic.InterruptEnableCtx3[reg_idx].set(plic.InterruptEnableCtx3[reg_idx].get() & !bit_mask)
      }
      2 => {
        plic.InterruptEnableCtx5[reg_idx].set(plic.InterruptEnableCtx5[reg_idx].get() & !bit_mask)
      }
      3 => {
        plic.InterruptEnableCtx7[reg_idx].set(plic.InterruptEnableCtx7[reg_idx].get() & !bit_mask)
      }
      _ => panic!(),
    }
  }

  fn fetch(&self) -> Option<(Interrupt, usize)> {
    let plic = &PLIC_MMIO;
    let core_id = core_id();
    let int = match core_id {
      0 => plic.InterruptClaimCompletionCtx1.get(),
      1 => plic.InterruptClaimCompletionCtx3.get(),
      2 => plic.InterruptClaimCompletionCtx5.get(),
      3 => plic.InterruptClaimCompletionCtx7.get(),
      _ => panic!(),
    } as usize;
    if int == 0 {
      None
    } else {
      Some((int, 0)) // plic is only for external interrupt, thus no source cpu
    }
  }

  fn finish(&self, int: Interrupt) {
    let plic = &PLIC_MMIO;
    let core_id = core_id();
    match core_id {
      0 => plic.InterruptClaimCompletionCtx1.set(int as u32),
      1 => plic.InterruptClaimCompletionCtx3.set(int as u32),
      2 => plic.InterruptClaimCompletionCtx5.set(int as u32),
      3 => plic.InterruptClaimCompletionCtx7.set(int as u32),
      _ => panic!(),
    };
  }
}

pub static INTERRUPT_CONTROLLER: Rv64InterruptController = Rv64InterruptController {
  ipi_mailboxes: Mutex::new([None; crate::MAX_CPU_NUMBER]),
};

pub type Interrupt = usize;

impl InterProcessorInterruptController for Rv64InterruptController {
  fn send_to_one(&self, irq: IPI, target: usize) {
    assert!(target != core_id());
    self.send_to_multiple(irq, 1usize << target);
  }

  fn send_to_multiple(&self, irq: IPI, target_mask: usize) {
    use super::sbi::*;
    let mut mailboxes = self.ipi_mailboxes.lock();
    for i in 0..crate::cpu_number() {
      if target_mask & (1usize << i) != 0 {
        let old = mailboxes[i].take();
        if let Some((ipi, core)) = old {
          trace!("dropping ipi {:?} from cpu {}", ipi, core);
        }
        mailboxes[i] = Some((irq, core_id()));
      }
    }
    drop(mailboxes);
    // we don't need hart_mask_base as we have far less than 64 harts
    let _ = sbi_call(SBI_EID_IPI, SBI_FID_IPI_SEND, target_mask, 0, 0);
  }
}

impl Rv64InterruptController {
  pub fn read_ipi_event(&self) -> Option<(IPI, usize)> {
    let mut mailbox = self.ipi_mailboxes.lock()[core_id()];
    mailbox.take()
  }
}