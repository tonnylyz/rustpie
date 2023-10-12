use alloc::collections::VecDeque;

use hardware::ns16550::*;

use rpsyscall::get_tid;
use rpsyscall::message::Message;
use tock_registers::interfaces::{Writeable, Readable};

const NS16550_MMIO_BASE: usize = 0x8_0000_0000 + 0x1000_0000;

static NS16550_MMIO: Ns16550Mmio = Ns16550Mmio::new(NS16550_MMIO_BASE);

fn init() {
  let ns16550 = &NS16550_MMIO;
  ns16550.ISR_FCR.write(ISR_FCR::EN_FIFO::Mode16550); // enable FIFO
  ns16550.IER_DLM.write(IER_DLM::IE_RHR::SET) // enable IE_RHR
}

fn getc() -> Option<u8> {
  let ns16550 = &NS16550_MMIO;
  if ns16550.LSR.is_set(LSR::RDR) {
    Some(ns16550.RHR_THR_DLL.get() as u8)
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
      let _ = rpsyscall::event_wait(rpabi::event::EVENT_INTERRUPT, 0xa);
    }
    msg.a = buf.pop_front().unwrap() as usize;
    client_tid = msg.reply_recv(client_tid).unwrap().0;
  }
}
