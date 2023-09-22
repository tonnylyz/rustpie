pub fn init() {}

fn send(c: u8) {
  let txfifo = (0xffff_ffff_0000_0000usize + 0x38000000) as *mut u32;
  unsafe {
    while txfifo.read_volatile() & 0x80000000 != 0 {}
    txfifo.write(c as u32);
  }
}

pub fn putc(c: u8) {
  if c == b'\n' {
    send(b'\r');
  }
  send(c);
}

pub fn getc() -> Option<u8> {
  let rxfifo = (0xffff_ffff_0000_0000usize + 0x38000004) as *mut u32;
  unsafe {
    let r = rxfifo.read_volatile();
    if r & 0x80000000 != 0 {
      None
    } else {
      if r == 0xa {
        Some(0xd)
      } else {
        Some(r as u8)
      }
    }
  }
}

