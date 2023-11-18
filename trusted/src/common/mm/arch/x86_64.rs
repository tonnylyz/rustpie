use super::PageAttribute;

#[derive(Clone, Copy, Debug)]
pub struct Entry(usize);

impl Entry {
  pub fn new(writable: bool, executable: bool, copy_on_write: bool, shared: bool) -> Self {
    Entry(0)
  }
}

impl PageAttribute for Entry {
  fn executable(&self) -> bool {
    todo!()
  }

  fn writable(&self) -> bool {
    todo!()
  }

  fn copy_on_write(&self) -> bool {
    todo!()
  }

  fn shared(&self) -> bool {
    todo!()
  }

  fn set_executable(&mut self, b: bool) {
    todo!()
  }

  fn set_writable(&mut self, b: bool) {
    todo!()
  }

  fn set_copy_on_write(&mut self, b: bool) {
    todo!()
  }

  fn set_shared(&mut self, b: bool) {
    todo!()
  }

  fn address(&self) -> usize {
    todo!()
  }

  fn set_address(&mut self, addr: usize) {
    todo!()
  }

  fn attribute(&self) -> usize {
    todo!()
  }

  fn set_attribute(&mut self, attr: usize) {
    todo!()
  }

  fn is_valid(&self) -> bool {
    todo!()
  }

  fn is_table(&self) -> bool {
    todo!()
  }

  fn is_page(&self) -> bool {
    todo!()
  }
}

pub fn query(va: usize) -> Option<Entry> { None }

#[allow(dead_code)]
pub fn traverse<F>(limit: usize, f: F) where F: Fn(usize, Entry) -> () { todo!() }