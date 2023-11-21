pub use heap::{virtual_alloc, virtual_free};
pub use heap::init as heap_init;
// pub use page_table::query;
// pub use page_table::traverse;

#[cfg(target_arch = "aarch64")]
#[path = "arch/aarch64.rs"]
mod arch;

#[cfg(target_arch = "riscv64")]
#[path = "arch/riscv64.rs"]
mod arch;

#[cfg(target_arch = "x86_64")]
#[path = "arch/x86_64.rs"]
mod arch;

mod heap;

pub fn default_page_attribute() -> usize {
  rpabi::syscall::mm::EntryAttribute::user_default().raw()
}

pub fn virt_to_phys(va: usize) -> usize {
  match arch::va_to_pa(va) {
    None => { 0 }
    Some(pa) => {
      pa | (va & (rpabi::PAGE_SIZE - 1))
    }
  }
}
