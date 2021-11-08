//! SD card slot access (in SPI mode) on Maix Go
use core::convert::TryInto;
use spin::Once;
use libtrusted::mm::{valloc, virt_to_phys};

#[path = "k210/mod.rs"]
mod k210;

const SD_CS: u32 = 3;
const SD_CS_GPIONUM: u8 = 7;
use k210_hal::pac::{Peripherals, SPI0};
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
use k210::board::sdcard::SDCard;
use k210::soc::spi::SPIImpl;
use k210::soc::dmac::DMAC;

static SDCARD: Once<SDCard> = Once::new();

fn init() {
  let p = Peripherals::take().unwrap();
  // sysctl::pll_set_freq(sysctl::pll::PLL0, 800_000_000).unwrap();
  // sysctl::pll_set_freq(sysctl::pll::PLL1, 300_000_000).unwrap();
  // sysctl::pll_set_freq(sysctl::pll::PLL2, 45_158_400).unwrap();

  fpioa::set_function(io::SPI0_SCLK, fpioa::function::SPI0_SCLK);
  fpioa::set_function(io::SPI0_MOSI, fpioa::function::SPI0_D0);
  fpioa::set_function(io::SPI0_MISO, fpioa::function::SPI0_D1);
  fpioa::set_function(io::SPI0_CS0, fpioa::function::gpiohs(SD_CS_GPIONUM));
  fpioa::set_io_pull(io::SPI0_CS0, fpioa::pull::DOWN); // GPIO output=pull down

  p.DMAC.configure();
  let sd = SDCARD.call_once(|| {
    sdcard::SDCard::new(SD_CS, SD_CS_GPIONUM,dma_channel::CHANNEL0)
  });
  let info = sd.init().unwrap();
  info!("card info {:?}", info);
}

pub fn server() {
  init();
  let sdcard = SDCARD.get().unwrap();
  let buf = valloc(1);
  unsafe { sdcard.read_sector(core::slice::from_raw_parts_mut(buf as *mut u8, 4096), 8); }
  info!("read ok to {:016x}", virt_to_phys(buf as usize));
  let content = unsafe { core::slice::from_raw_parts(buf, 4096) };
  for i in 0..4096 {
    print!("{:02x} ", content[i]);
    if (i + 1) % 16 == 0 {
      println!();
    }
  }
  loop {}
}
