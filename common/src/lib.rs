#![no_std]

cfg_if::cfg_if! {
  if #[cfg(target_arch = "aarch64")] {
    pub const CONFIG_RECURSIVE_PAGE_TABLE_BTM: usize = 0x3f_c000_0000;
  } else if #[cfg(target_arch = "riscv64")] {
    pub const CONFIG_RECURSIVE_PAGE_TABLE_BTM: usize = 0; // Note: not used
    pub const CONFIG_READ_ONLY_LEVEL_3_PAGE_TABLE_BTM: usize = 0x3f_c000_0000;
    pub const CONFIG_READ_ONLY_LEVEL_2_PAGE_TABLE_BTM: usize = 0x3f_c000_0000 - 0x20_0000;
    pub const CONFIG_READ_ONLY_LEVEL_1_PAGE_TABLE_BTM: usize = 0x3f_c000_0000 - 0x20_0000 - 0x1000; // 4 KB
  } else {
    compile_error!("unsupported target_arch");
  }
}

pub const CONFIG_USER_LIMIT: usize = 0x3f_a000_0000;
pub const CONFIG_USER_STACK_TOP: usize = 0x3f_8000_0000;
pub const CONFIG_EXCEPTION_STACK_TOP: usize = 0x3f_8000_1000;
pub const CONFIG_HEAP_BTM: usize = 0x10_0000_0000;