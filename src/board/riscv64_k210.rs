use alloc::vec::Vec;
use core::fmt::Write;
use core::ops::Range;
use crate::lib::device::Device;
use crate::lib::interrupt::InterruptController;
use crate::lib::traits::ArchTrait;

pub const BOARD_CORE_NUMBER: usize = 1;

pub const BOARD_NORMAL_MEMORY_RANGE: Range<usize> = 0x8000_0000..0x8060_0000;
pub const BOARD_DEVICE_MEMORY_RANGE: Range<usize> = 0x0000_0000..0x8000_0000;

#[path = "../driver/k210/mod.rs"]
mod k210;

pub fn init() {
  const SD_CS: u32 = 3;
  const SD_CS_GPIONUM: u8 = 7;
  use k210_hal::pac::Peripherals;
  use k210_hal::serial::SerialExt;
  use k210_hal::time::U32Ext;
  use k210_hal::stdout::Stdout;
  use k210::soc::sysctl;
  use k210::soc::fpioa;
  use k210::soc::spi::SPIExt;
  use k210::soc::dmac::{dma_channel, DMACExt};
  use k210::soc::sleep::usleep;
  use k210::board::def::io;
  use k210::board::sdcard;

  let p = Peripherals::take().unwrap();
  // sysctl::pll_set_freq(sysctl::pll::PLL0, 800_000_000).unwrap();
  // sysctl::pll_set_freq(sysctl::pll::PLL1, 300_000_000).unwrap();
  // sysctl::pll_set_freq(sysctl::pll::PLL2, 45_158_400).unwrap();

  fpioa::set_function(io::SPI0_SCLK, fpioa::function::SPI0_SCLK);
  fpioa::set_function(io::SPI0_MOSI, fpioa::function::SPI0_D0);
  fpioa::set_function(io::SPI0_MISO, fpioa::function::SPI0_D1);
  fpioa::set_function(io::SPI0_CS0, fpioa::function::gpiohs(SD_CS_GPIONUM));
  fpioa::set_io_pull(io::SPI0_CS0, fpioa::pull::DOWN); // GPIO output=pull down

  let dmac = p.DMAC.configure();
  let spi = p.SPI0.constrain();
  let sd = sdcard::SDCard::new(spi, SD_CS, SD_CS_GPIONUM, &dmac, dma_channel::CHANNEL0);
  let info = sd.init().unwrap();
  println!("card info {:?}", info);
}

pub fn init_per_core() {
  crate::driver::timer::init();
  crate::arch::Arch::exception_init();
  crate::driver::INTERRUPT_CONTROLLER.init();
}


pub fn launch_other_cores() {
  extern "C" {
    fn KERNEL_ENTRY();
  }
  // let _ = crate::driver::hsm::hart_start(1, (KERNEL_ENTRY as usize).kva2pa(), 0);
}

#[no_mangle]
pub unsafe extern "C" fn hart_spin(core_id: usize) {
  crate::main(core_id);
}

pub fn devices() -> Vec<Device> {
  vec![
    Device::new(
      "GPIOHS",
      vec![
        0x3800_1000..0x3800_2000
      ],
      vec![]),
    Device::new(
      "SPI0",
      vec![
        0x5200_0000..0x5200_1000
      ],
      vec![]),
    Device::new(
      "DMAC",
      vec![
        0x5000_0000..0x5000_1000
      ],
      vec![]),
    Device::new(
      "SYSCTL",
      vec![
        0x5044_0000..0x5044_1000
      ],
      vec![]),
  ]
}
