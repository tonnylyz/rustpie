use alloc::collections::VecDeque;

use hardware::pl011::Pl011Mmio;

use rpsyscall::get_tid;
use rpsyscall::message::Message;
use tock_registers::interfaces::{Writeable, Readable};

const UART_BASE: usize = 0x8_0000_0000 + 0x900_0000;

static PL011_MMIO: Pl011Mmio = Pl011Mmio::new(UART_BASE);

const UART_FR_RXFE: u32 = 1 << 4;

fn init() {
  let pl011 = &PL011_MMIO;
  pl011.LineControl.set(1u32 << 4); // enable FIFO
  let int_mask = pl011.IntMaskSetClr.get();
  pl011.IntMaskSetClr.set(int_mask | (1u32 << 4)); // enable UARTRXINTR
}

fn getc() -> Option<u8> {
  let pl011 = &PL011_MMIO;
  if pl011.Flag.get() & UART_FR_RXFE == 0 {
    Some((pl011.Data.get() & 0xff) as u8)
  } else {
    None
  }
}

fn drain_rx_fifo(buf: &mut VecDeque<u8>) {
  loop {
    match getc() {
      Some(c) => {
        buf.push_back(c);
      },
      None => break
    }
  }
}

pub fn server() {
  info!("server started t{}",  get_tid());
  init();
  rpsyscall::server_register(rpabi::server::SERVER_TERMINAL).unwrap();
  let mut buf = VecDeque::new();
  let mut client_tid;
  client_tid = Message::receive().unwrap().0;
  loop {
    let mut msg = rpsyscall::message::Message::default();
    loop {
      drain_rx_fifo(&mut buf);
      if !buf.is_empty() {
        break;
      }
      let _ = rpsyscall::event_wait(rpabi::event::EVENT_INTERRUPT, 0x1 + 32);
    }
    msg.a = buf.pop_front().unwrap() as usize;
    client_tid = msg.reply_recv(client_tid).unwrap().0;
  }
}
