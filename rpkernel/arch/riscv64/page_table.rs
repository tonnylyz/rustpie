use alloc::vec::Vec;
use hardware::mmu::riscv64_mmu::vm_descriptor::*;
use rpabi::syscall::mm::EntryAttribute;

use crate::arch::*;
use crate::kernel::traits::*;
use crate::mm::page_table::{Entry, Error, PageTableTrait};
use crate::mm::PhysicalFrame;

pub const PAGE_TABLE_L1_SHIFT: usize = 30;
pub const PAGE_TABLE_L2_SHIFT: usize = 21;
pub const PAGE_TABLE_L3_SHIFT: usize = 12;

#[derive(Debug)]
pub struct Riscv64PageTable {
  directory_kva: usize,
}

#[repr(transparent)]
#[derive(Copy, Clone, Debug)]
pub struct Riscv64PageTableEntry(usize);

impl Riscv64PageTableEntry {
  fn from_pa(pa: usize) -> Self {
    Riscv64PageTableEntry((pa >> 12) << 10)
  }

  fn to_pa(&self) -> usize {
    (self.0 >> 10) << 12
  }

  fn to_kva(&self) -> usize {
    self.to_pa().pa2kva()
  }

  fn valid(&self) -> bool {
    // V and NOT RWX
    self.0 & 0b1 != 0
  }

  fn entry(&self, index: usize) -> Self {
    let addr = self.to_kva() + index * MACHINE_SIZE;
    unsafe { Riscv64PageTableEntry((addr as *const usize).read_volatile()) }
  }

  fn set_entry(&self, index: usize, value: Self) {
    let addr = self.to_kva() + index * MACHINE_SIZE;
    unsafe { (addr as *mut usize).write_volatile(value.0) }
  }

  fn make_table(frame_pa: usize) -> Self {
    Riscv64PageTableEntry(
      (TABLE_DESCRIPTOR::NEXT_LEVEL_TABLE_PPN.val((frame_pa >> PAGE_SHIFT) as u64)
      // NOTE from priv spec 1.12:
      // For non-leaf PTEs, the D, A, and U bits are reserved for future standard use. Until their use is
      // defined by a standard extension, they must be cleared by software for forward compatibility.
      + TABLE_DESCRIPTOR::DIRTY::False
      + TABLE_DESCRIPTOR::ACCESSED::False
      + TABLE_DESCRIPTOR::USER::False
        + TABLE_DESCRIPTOR::VALID::True)
        .value as usize,
    )
  }
}

trait Index {
  fn l1x(&self) -> usize;
  fn l2x(&self) -> usize;
  fn l3x(&self) -> usize;
}

impl Index for usize {
  fn l1x(&self) -> usize {
    self >> PAGE_TABLE_L1_SHIFT & (PAGE_SIZE / MACHINE_SIZE - 1)
  }
  fn l2x(&self) -> usize {
    self >> PAGE_TABLE_L2_SHIFT & (PAGE_SIZE / MACHINE_SIZE - 1)
  }
  fn l3x(&self) -> usize {
    self >> PAGE_TABLE_L3_SHIFT & (PAGE_SIZE / MACHINE_SIZE - 1)
  }
}

impl core::convert::From<Riscv64PageTableEntry> for Entry {
  fn from(u: Riscv64PageTableEntry) -> Self {
    use tock_registers::*;
    let reg = LocalRegisterCopy::<u64, PAGE_DESCRIPTOR::Register>::new(u.0 as u64);
    Entry::new(
      EntryAttribute::new(
        reg.is_set(PAGE_DESCRIPTOR::W),
        reg.is_set(PAGE_DESCRIPTOR::USER),
        false, // riscv do not has bits indicating device memory
        false, // reg.is_set(PAGE_DESCRIPTOR::X)  && SUM bit in sstatus
        reg.is_set(PAGE_DESCRIPTOR::X),
        reg.is_set(PAGE_DESCRIPTOR::COW),
        reg.is_set(PAGE_DESCRIPTOR::LIB),
      ),
      (reg.read(PAGE_DESCRIPTOR::OUTPUT_PPN) as usize) << PAGE_SHIFT,
    )
  }
}

impl core::convert::From<Entry> for Riscv64PageTableEntry {
  fn from(pte: Entry) -> Self {
    let r = Riscv64PageTableEntry(
      (if pte.attribute().u_shared() {
        PAGE_DESCRIPTOR::LIB::True
      } else {
        PAGE_DESCRIPTOR::LIB::False
      } + if pte.attribute().copy_on_write() {
        PAGE_DESCRIPTOR::COW::True
      } else {
        PAGE_DESCRIPTOR::COW::False
      } + if pte.attribute().u_executable() {
        PAGE_DESCRIPTOR::X::True
      } else {
        PAGE_DESCRIPTOR::X::False
      } + if pte.attribute().u_readable() {
        PAGE_DESCRIPTOR::R::True
      } else {
        PAGE_DESCRIPTOR::R::False
      } + if pte.attribute().writable() {
        PAGE_DESCRIPTOR::W::True
      } else {
        PAGE_DESCRIPTOR::W::False
      } + PAGE_DESCRIPTOR::DIRTY::True
        + PAGE_DESCRIPTOR::ACCESSED::True
        + PAGE_DESCRIPTOR::VALID::True
        + PAGE_DESCRIPTOR::USER::True
        + PAGE_DESCRIPTOR::OUTPUT_PPN.val((pte.ppn()) as u64))
      .value as usize,
    );
    r
  }
}

impl Riscv64PageTable {
  fn map_kernel_gigabyte_page(&self, va: usize, pa: usize) {
    let l1x = va.l1x();
    let directory = Riscv64PageTableEntry::from_pa(self.directory_kva.kva2pa());
    // same as mapping in `start.S`
    directory.set_entry(
      l1x,
      Riscv64PageTableEntry(
        (PAGE_DESCRIPTOR::OUTPUT_PPN.val((pa >> PAGE_SHIFT) as u64)
        + PAGE_DESCRIPTOR::DIRTY::True
        + PAGE_DESCRIPTOR::ACCESSED::True
        + PAGE_DESCRIPTOR::USER::False
        + PAGE_DESCRIPTOR::GLOBAL::True // Kernel is always global
        + PAGE_DESCRIPTOR::X::True
        + PAGE_DESCRIPTOR::W::True
        + PAGE_DESCRIPTOR::R::True
        + PAGE_DESCRIPTOR::VALID::True)
          .value as usize,
      ),
    );
  }
}

impl PageTableTrait for Riscv64PageTable {
  fn new(directory_kva: usize, table_frames: &mut Vec<PhysicalFrame>) -> Self {
    let r = Riscv64PageTable { directory_kva };
    r.map_kernel_gigabyte_page(0xffff_ffff_0000_0000, 0x0000_0000);
    r.map_kernel_gigabyte_page(0xffff_ffff_4000_0000, 0x4000_0000);
    r.map_kernel_gigabyte_page(0xffff_ffff_8000_0000, 0x8000_0000);
    r.map_kernel_gigabyte_page(0xffff_ffff_c000_0000, 0xc000_0000);

    r.map(
      rpabi::CONFIG_READ_ONLY_LEVEL_1_PAGE_TABLE_BTM,
      directory_kva.kva2pa(),
      EntryAttribute::user_readonly(),
      table_frames,
    )
    .unwrap();
    r
  }

  fn map(
    &self,
    va: usize,
    pa: usize,
    attr: EntryAttribute,
    table_frames: &mut Vec<PhysicalFrame>,
  ) -> Result<(), Error> {
    let directory = Riscv64PageTableEntry::from_pa(self.directory_kva.kva2pa());
    let mut l1e = directory.entry(va.l1x());
    if !l1e.valid() {
      let frame = crate::mm::page_pool::page_alloc()?;
      frame.zero();
      l1e = Riscv64PageTableEntry::make_table(frame.pa());
      table_frames.push(frame);
      if va <= rpabi::CONFIG_READ_ONLY_LEVEL_1_PAGE_TABLE_BTM {
        self.map(
          rpabi::CONFIG_READ_ONLY_LEVEL_2_PAGE_TABLE_BTM + va.l1x() * PAGE_SIZE,
          l1e.to_pa(),
          EntryAttribute::user_readonly(),
          table_frames
        )?;
      }
      directory.set_entry(va.l1x(), l1e);
    }
    let mut l2e = l1e.entry(va.l2x());
    if !l2e.valid() {
      let frame = crate::mm::page_pool::page_alloc()?;
      frame.zero();
      l2e = Riscv64PageTableEntry::make_table(frame.pa());
      table_frames.push(frame);
      if va <= rpabi::CONFIG_READ_ONLY_LEVEL_1_PAGE_TABLE_BTM {
        self.map(
          rpabi::CONFIG_READ_ONLY_LEVEL_3_PAGE_TABLE_BTM
            + va.l1x() * PAGE_SIZE * (PAGE_SIZE / MACHINE_SIZE)
            + va.l2x() * PAGE_SIZE,
          l2e.to_pa(),
          EntryAttribute::user_readonly(),
          table_frames
        )?;
      }
      l1e.set_entry(va.l2x(), l2e);
    }
    l2e.set_entry(va.l3x(), Riscv64PageTableEntry::from(Entry::new(attr, pa)));
    crate::arch::Arch::invalidate_tlb();
    Ok(())
  }

  fn unmap(&self, va: usize) {
    let directory = Riscv64PageTableEntry::from_pa(self.directory_kva.kva2pa());
    let l1e = directory.entry(va.l1x());
    assert!(l1e.valid());
    let l2e = l1e.entry(va.l2x());
    assert!(l2e.valid());
    l2e.set_entry(va.l3x(), Riscv64PageTableEntry(0));
  }

  fn lookup_page(&self, va: usize) -> Option<Entry> {
    let directory = Riscv64PageTableEntry::from_pa(self.directory_kva.kva2pa());
    let l1e = directory.entry(va.l1x());
    if !l1e.valid() {
      return None;
    }
    let l2e = l1e.entry(va.l2x());
    if !l2e.valid() {
      return None;
    }
    let l3e = l2e.entry(va.l3x());
    if l3e.valid() {
      Some(Entry::from(l3e))
    } else {
      None
    }
  }

  fn recursive_map(&self, _va: usize) {
    // riscv64 can't do recursive_map
  }
}
