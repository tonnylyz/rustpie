//! SD card slot access (in SPI mode) on Maix Go
use k210_hal::pac::Peripherals;

use k210::board::def::io;
use k210::board::sdcard;
use k210::soc::dmac::{dma_channel, DMACExt};
use k210::soc::fpioa;
use k210::soc::spi::SPIExt;
use rpsyscall::get_tid;

#[path = "k210/mod.rs"]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[allow(unused_imports)]
#[allow(dead_code)]
mod k210;

const SD_CS: u32 = 3;
const SD_CS_GPIONUM: u8 = 7;

pub fn server() {
  fpioa::set_function(io::SPI0_SCLK, fpioa::function::SPI0_SCLK);
  fpioa::set_function(io::SPI0_MOSI, fpioa::function::SPI0_D0);
  fpioa::set_function(io::SPI0_MISO, fpioa::function::SPI0_D1);
  fpioa::set_function(io::SPI0_CS0, fpioa::function::gpiohs(SD_CS_GPIONUM));
  fpioa::set_io_pull(io::SPI0_CS0, fpioa::pull::DOWN); // GPIO output=pull down

  let p = Peripherals::take().unwrap();
  let dmac = p.DMAC.configure();
  let spi = p.SPI0.constrain();
  let sd = sdcard::SDCard::new(spi, SD_CS, SD_CS_GPIONUM, &dmac, dma_channel::CHANNEL0);
  let info = sd.init().unwrap();
  info!("{:?}", info);
  let blk_size = info.CardCapacity as usize;
  info!("server started t{}", get_tid());
  rpsyscall::server_register(rpabi::server::SERVER_BLK).unwrap();

  loop {
    let (client_tid, msg) = rpsyscall::message::Message::receive().unwrap();
    // info!("recv {:x?}", (client_tid, msg));
    if msg.d == 0 || msg.d == 1 {
      let sector = msg.a;
      let count = msg.b;
      let buf = msg.c;
      if msg.d == 0 {
        // Operation::Read
        let buf = unsafe {
          core::slice::from_raw_parts_mut(buf as *mut u8, count * 512)
        };
        let r = sd.read_sector(buf, sector as u32);
        if r.is_ok() {
          let msg = rpsyscall::message::Message::default();
          let _ = msg.send_to(client_tid);
        } else {
          error!("read_sector error");
        }
      } else {
        // Operation::Write
        let buf = unsafe {
          core::slice::from_raw_parts(buf as *mut u8, count * 512)
        };
        let r = sd.write_sector(buf, sector as u32);
        if r.is_ok() {
          let msg = rpsyscall::message::Message::default();
          let _ = msg.send_to(client_tid);
        } else {
          error!("write_sector error");
        }
      }
    } else if msg.d == 2 {
      let mut msg = rpsyscall::message::Message::default();
      msg.a = blk_size;
      let _ = msg.send_to(client_tid);
    } else {
      let mut msg = rpsyscall::message::Message::default();
      msg.a = 1;
      let _ = msg.send_to(client_tid);
    }
  }
}
