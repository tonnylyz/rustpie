pub const IPC_PCB_SIZE: usize = 32;
pub const IPC_LIST_BTM: usize = 0x3f_a000_0000;

pub const USER_LIMIT: usize = 0x3f_a000_0000;

pub const EXCEPTION_STACK_TOP: usize = 0x3f_8000_1000;

pub const TRAVERSE_LIMIT: usize = 0x3f_8000_0000;
pub const STACK_TOP: usize = 0x3f_8000_0000;

pub const HEAP_BTM: usize = 0x10_0000_0000;

pub const PAGE_SIZE: usize = 4096;

pub const PAGE_TABLE_L1_SHIFT: usize = 30;
pub const PAGE_TABLE_L2_SHIFT: usize = 21;
pub const PAGE_TABLE_L3_SHIFT: usize = 12;

pub const WORD_SHIFT: usize = 3;
pub const WORD_SIZE: usize = 8;
