use core::ops::Range;

use crate::arch::{ArchTrait, CoreTrait, PAGE_SIZE};
use crate::lib::current_core;
use crate::lib::interrupt::InterruptController;
use crate::driver::gic::INT_TIMER;

pub const BOARD_CORE_NUMBER: usize = 4;
pub const BOARD_PHYSICAL_ADDRESS_LIMIT: usize = 0x8000_0000;
pub const BOARD_NORMAL_MEMORY_RANGE: Range<usize> = 0x4000_0000..0x8000_0000;
pub const BOARD_DEVICE_MEMORY_RANGE: Range<usize> = 0x0000_0000..0x4000_0000;
pub const BOARD_PHYSICAL_ENTRY: u64 = 0x40080000;


pub fn init() {
    crate::driver::uart::init();
    crate::driver::common::virtio_blk::init();
}

pub fn init_per_core() {
    use cortex_a::regs::*;
    DAIF.write(DAIF::I::Masked);
    crate::arch::Arch::exception_init();
    crate::driver::INTERRUPT_CONTROLLER.init();
    crate::driver::INTERRUPT_CONTROLLER.enable(INT_TIMER);
    crate::driver::timer::init();
    current_core().create_idle_thread();
}

pub fn launch_other_cores() {
    let core_id = crate::arch::Arch::core_id();
    for i in 0..BOARD_CORE_NUMBER {
        if i != core_id {
            crate::driver::psci::cpu_on(i as u64, BOARD_PHYSICAL_ENTRY, 0);
        }
    }
}
