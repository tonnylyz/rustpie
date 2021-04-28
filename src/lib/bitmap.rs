use core::mem::size_of;

pub struct BitMap<const S: usize>([usize; S]);

impl<const S: usize> BitMap<S> {
  pub const fn new() -> Self {
    BitMap::<S>([0; S])
  }

  pub fn set(&mut self, index: usize) {
    let i = index / size_of::<usize>();
    let shift = index % size_of::<usize>();
    self.0[i] |= 1usize << shift;
  }

  pub fn clear(&mut self, index: usize) {
    let i = index / size_of::<usize>();
    let shift = index % size_of::<usize>();
    self.0[i] &= !(1usize << shift);
  }

  #[allow(dead_code)]
  pub fn is_set(&self, index: usize) -> bool {
    let i = index / size_of::<usize>();
    let shift = index % size_of::<usize>();
    (self.0[i] >> shift) & 0b1 == 0b1
  }

  pub fn alloc(&mut self) -> usize {
    for i in 0..(S / size_of::<usize>()) {
      let atom = self.0[i];
      if atom == usize::MAX {
        continue;
      }
      for shift in 0..size_of::<usize>() {
        if (atom >> shift) & 0b1 == 0 {
          let index = i * size_of::<usize>() + shift;
          self.set(index);
          return index;
        }
      }
    }
    panic!("bitmap: out of zero bit");
  }
}