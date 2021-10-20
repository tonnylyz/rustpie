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
pub const CONFIG_USER_STACK_BTM: usize = 0x3f_0000_0000;
pub const CONFIG_EXCEPTION_STACK_TOP: usize = 0x3f_8000_1000;
pub const CONFIG_HEAP_BTM: usize = 0x10_0000_0000;

pub const CONFIG_ELF_IMAGE: usize = 0x8000_0000;

pub const PAGE_SIZE: usize = 4096;

pub const PAGE_TABLE_L1_SHIFT: usize = 30;
pub const PAGE_TABLE_L2_SHIFT: usize = 21;
pub const PAGE_TABLE_L3_SHIFT: usize = 12;

pub const WORD_SHIFT: usize = 3;
pub const WORD_SIZE: usize = 1 << WORD_SHIFT;

pub mod syscall {
  pub const SYS_NULL: usize = 0;
  pub const SYS_PUTC: usize = 1;
  pub const SYS_GET_ASID: usize = 2;
  pub const SYS_GET_TID: usize = 3;
  pub const SYS_THREAD_YIELD: usize = 4;
  pub const SYS_THREAD_DESTROY: usize = 5;
  pub const SYS_EVENT_WAIT: usize = 6;
  pub const SYS_MEM_ALLOC: usize = 7;
  pub const SYS_MEM_MAP: usize = 8;
  pub const SYS_MEM_UNMAP: usize = 9;
  pub const SYS_ADDRESS_SPACE_ALLOC: usize = 10;
  pub const SYS_THREAD_ALLOC: usize = 11;
  pub const SYS_THREAD_SET_STATUS: usize = 12;
  pub const SYS_ADDRESS_SPACE_DESTROY: usize = 13;
  pub const SYS_ITC_RECV: usize = 15;
  pub const SYS_ITC_SEND: usize = 16;
  pub const SYS_ITC_CALL: usize = 17;
  pub const SYS_SERVER_REGISTER: usize = 19;
  pub const SYS_SERVER_TID: usize = 20;
  pub const SYS_SET_EXCEPTION_HANDLER: usize = 21;
  pub const SYS_GETC: usize = 22;
  pub const SYS_MAX: usize = 23;

  pub mod error {
    pub const ERROR_INVARG: usize = 1;
    pub const ERROR_OOM: usize = 2;
    pub const ERROR_MEM_NOT_MAP: usize = 3;
    pub const ERROR_INTERNAL: usize = 4;
    pub const ERROR_DENIED: usize = 5;
    pub const ERROR_HOLD_ON: usize = 6;
    pub const ERROR_OOR: usize = 7;
    pub const ERROR_PANIC: usize = 8;
  }
}

pub mod server {
  pub const SERVER_VIRTIO_BLK: usize = 0;
  pub const SERVER_REDOX_FS: usize = 1;
  pub const SERVER_TERMINAL: usize = 2;
  pub const SERVER_MM: usize = 3;
  pub const SERVER_PM: usize = 4;
  pub const SERVER_TEST: usize = 5;
}

pub mod thread {
  pub const THREAD_STATUS_RUNNABLE: usize = 1;
  pub const THREAD_STATUS_NOT_RUNNABLE: usize = 2;
}

pub mod event {
  pub const EVENT_INTERRUPT: usize = 1;
  pub const EVENT_THREAD_EXIT: usize = 2;
}