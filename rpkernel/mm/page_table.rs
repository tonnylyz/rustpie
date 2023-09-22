use core::fmt::{Display, Formatter};

use crate::arch::{AddressSpaceId, PAGE_SHIFT};
use crate::mm::Frame;

pub trait PageTableEntryAttrTrait {
  fn writable(&self) -> bool;
  fn k_executable(&self) -> bool;
  fn u_executable(&self) -> bool;
  fn u_readable(&self) -> bool;
  fn u_copy_on_write(&self) -> bool;
  fn u_shared(&self) -> bool;
  fn device(&self) -> bool;
  fn copy_on_write(&self) -> bool;

  fn new(writable: bool,
         user: bool,
         device: bool,
         k_executable: bool,
         u_executable: bool,
         copy_on_write: bool,
         shared: bool) -> Self;
  fn kernel_device() -> Self;
  fn user_default() -> Self;
  fn user_readonly() -> Self;
  fn user_executable() -> Self;
  fn user_data() -> Self;
  fn user_device() -> Self;
  fn filter(&self) -> Self;
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct EntryAttribute {
  writable: bool,
  user: bool,
  device: bool,
  k_executable: bool,
  u_executable: bool,
  copy_on_write: bool,
  shared: bool,
}

impl PageTableEntryAttrTrait for EntryAttribute {
  fn writable(&self) -> bool {
    self.writable
  }

  fn k_executable(&self) -> bool {
    self.k_executable
  }

  fn u_executable(&self) -> bool {
    self.u_executable
  }

  fn u_readable(&self) -> bool {
    self.user
  }

  fn u_copy_on_write(&self) -> bool {
    self.copy_on_write
  }

  fn u_shared(&self) -> bool {
    self.shared
  }

  fn device(&self) -> bool {
    self.device
  }

  fn copy_on_write(&self) -> bool {
    self.copy_on_write
  }

  fn new(writable: bool, user: bool, device: bool, k_executable: bool, u_executable: bool, copy_on_write: bool, shared: bool) -> Self {
    EntryAttribute {
      writable,
      user,
      device,
      k_executable,
      u_executable,
      copy_on_write,
      shared,
    }
  }

  fn kernel_device() -> Self {
    EntryAttribute {
      writable: true,
      user: false,
      device: true,
      k_executable: false,
      u_executable: false,
      copy_on_write: false,
      shared: false,
    }
  }

  fn user_default() -> Self {
    EntryAttribute {
      writable: true,
      user: true,
      device: false,
      k_executable: false,
      u_executable: true,
      copy_on_write: false,
      shared: false,
    }
  }

  fn user_readonly() -> Self {
    EntryAttribute {
      writable: false,
      user: true,
      device: false,
      k_executable: false,
      u_executable: false,
      copy_on_write: false,
      shared: false,
    }
  }

  fn user_executable() -> Self {
    EntryAttribute {
      writable: false,
      user: true,
      device: false,
      k_executable: false,
      u_executable: true,
      copy_on_write: false,
      shared: false,
    }
  }

  fn user_data() -> Self {
    EntryAttribute {
      writable: true,
      user: true,
      device: false,
      k_executable: false,
      u_executable: false,
      copy_on_write: false,
      shared: false,
    }
  }

  fn user_device() -> Self {
    EntryAttribute {
      writable: true,
      user: true,
      device: true,
      k_executable: false,
      u_executable: false,
      copy_on_write: false,
      shared: false,
    }
  }

  fn filter(&self) -> Self {
    EntryAttribute {
      writable: self.writable,
      user: true,
      device: false,
      k_executable: false,
      u_executable: self.u_executable,
      copy_on_write: self.copy_on_write,
      shared: self.shared,
    }
  }
}

#[derive(Copy, Clone, Debug)]
pub struct Entry {
  attribute: EntryAttribute,
  pa: usize,
}

impl Entry {
  pub fn new(attribute: EntryAttribute, pa: usize) -> Self {
    Entry {
      attribute,
      pa,
    }
  }
  pub fn attribute(&self) -> EntryAttribute {
    self.attribute
  }
  pub fn pa(&self) -> usize {
    self.pa
  }
  pub fn ppn(&self) -> usize { self.pa >> PAGE_SHIFT }
}

impl Display for Entry {
  fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), core::fmt::Error> {
    write!(f, "PageTableEntry [{:016x}] {:?}", self.pa, self.attribute)
  }
}

pub type Error = usize;

pub trait PageTableTrait {
  fn new(directory: crate::mm::PhysicalFrame) -> Self;
  fn base_pa(&self) -> usize;
  fn map(&self, va: usize, pa: usize, attr: EntryAttribute) -> Result<(), Error>;
  fn unmap(&self, va: usize);
  fn insert_page(&self, va: usize, user_frame: crate::mm::Frame, attr: EntryAttribute) -> Result<(), Error>;
  fn lookup_page(&self, va: usize) -> Option<Entry>;
  fn lookup_user_page(&self, va: usize) -> Option<Frame>;
  fn remove_page(&self, va: usize) -> Result<(), Error>;
  fn recursive_map(&self, va: usize);

  fn install_user_page_table(base: usize, asid: AddressSpaceId);
}
