use crate::constants::PAGE_SIZE;

pub trait Address {
  fn va2pa(&self) -> usize;
}

pub trait EntryLike {
  fn executable(&self) -> bool;
  fn writable(&self) -> bool;
  fn copy_on_write(&self) -> bool;
  fn shared(&self) -> bool;

  fn set_executable(&mut self, b: bool);
  fn set_writable(&mut self, b: bool);
  fn set_copy_on_write(&mut self, b: bool);
  fn set_shared(&mut self, b: bool);

  fn address(&self) -> usize;
  fn set_address(&mut self, addr: usize);

  fn attribute(&self) -> usize;
  fn set_attribute(&mut self, attr: usize);

  fn is_valid(&self) -> bool;
  fn is_table(&self) -> bool;
  fn is_page(&self) -> bool;
}

impl Address for usize {

  fn va2pa(&self) -> usize {
    match crate::arch::page_table::query(*self) {
      None => { 0 }
      Some(pte) => {
        pte.address() | (*self & (PAGE_SIZE - 1))
      }
    }
  }
}
