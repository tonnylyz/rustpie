use alloc::collections::VecDeque;

use hardware::ns16550::*;

use rpsyscall::get_tid;
use rpsyscall::message::Message;
use tock_registers::interfaces::{Readable, Writeable};

fn init(ns16550: &Ns16550Mmio) {
  ns16550.ISR_FCR.write(ISR_FCR::EN_FIFO::Mode16550); // enable FIFO
  ns16550.IER_DLM.write(IER_DLM::IE_RHR::SET) // enable IE_RHR
}

fn getc(ns16550: &Ns16550Mmio) -> Option<u8> {
  if ns16550.LSR.is_set(LSR::RDR) {
    Some(ns16550.RHR_THR_DLL.get() as u8)
  } else {
    None
  }
}

fn drain_rx_fifo(ns16550: &Ns16550Mmio, buf: &mut VecDeque<u8>) {
  loop {
    match getc(ns16550) {
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
  let ns16550 = Ns16550Mmio::new(base_addr);
  init(&ns16550);
  rpsyscall::server_register(rpabi::server::SERVER_TERMINAL).unwrap();
  let mut buf = VecDeque::new();
  let mut client_tid;
  client_tid = Message::receive().unwrap().0;
  loop {
    let mut msg = rpsyscall::message::Message::default();
    loop {
      drain_rx_fifo(&ns16550, &mut buf);
      if !buf.is_empty() {
        break;
      }
      let _ = rpsyscall::event_wait(rpabi::event::EVENT_INTERRUPT, irq_num);
    }
    msg.a = buf.pop_front().unwrap() as usize;
    client_tid = msg.reply_recv(client_tid).unwrap().0;
  }
}
