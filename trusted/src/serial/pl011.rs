use alloc::collections::VecDeque;

use hardware::pl011::Pl011Mmio;

use rpsyscall::get_tid;
use rpsyscall::message::Message;
use tock_registers::interfaces::{Readable, Writeable};

const UART_FR_RXFE: u32 = 1 << 4;

fn init(pl011: &Pl011Mmio) {
  pl011.LineControl.set(1u32 << 4); // enable FIFO
  let int_mask = pl011.IntMaskSetClr.get();
  pl011.IntMaskSetClr.set(int_mask | (1u32 << 4)); // enable UARTRXINTR
}

fn getc(pl011: &Pl011Mmio) -> Option<u8> {
  if pl011.Flag.get() & UART_FR_RXFE == 0 {
    Some((pl011.Data.get() & 0xff) as u8)
  } else {
    None
  }
}

fn drain_rx_fifo(pl011: &Pl011Mmio, buf: &mut VecDeque<u8>) {
  loop {
    match getc(pl011) {
      Some(c) => {
        buf.push_back(c);
      }
      None => break,
    }
  }
}

pub fn server(base_addr: usize, irq_num: usize) {
  info!("server started t{}", get_tid());
  let base_addr = base_addr + rpabi::platform::USER_SPACE_DRIVER_MMIO_OFFSET;
  let pl011 = Pl011Mmio::new(base_addr);
  init(&pl011);
  rpsyscall::server_register(rpabi::server::SERVER_TERMINAL).unwrap();
  let mut buf = VecDeque::new();
  let mut client_tid;
  client_tid = Message::receive().unwrap().0;
  loop {
    let mut msg = rpsyscall::message::Message::default();
    loop {
      drain_rx_fifo(&pl011, &mut buf);
      if !buf.is_empty() {
        break;
      }
      let _ = rpsyscall::event_wait(rpabi::event::EVENT_INTERRUPT, irq_num);
    }
    msg.a = buf.pop_front().unwrap() as usize;
    client_tid = msg.reply_recv(client_tid).unwrap().0;
  }
}
