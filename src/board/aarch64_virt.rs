use core::ops::Range;

use crate::arch::{Address, ArchTrait, CoreTrait};
use crate::lib::current_core;
use crate::lib::page_table::{EntryAttribute, PageTableEntryAttrTrait, PageTableTrait};

pub const BOARD_CORE_NUMBER: usize = 4;
pub const BOARD_PHYSICAL_ADDRESS_LIMIT: usize = 0x1_0000_0000;
pub const BOARD_NORMAL_MEMORY_RANGE: Range<usize> = 0x4000_0000..0x1_0000_0000;
pub const BOARD_DEVICE_MEMORY_RANGE: Range<usize> = 0x0000_0000..0x4000_0000;

pub fn init() {
    crate::driver::uart::init();
}

pub fn init_per_core() {
    crate::driver::timer::init();
    crate::arch::Arch::exception_init();

    current_core().create_idle_thread();
}

pub fn launch_other_cores() {

}
