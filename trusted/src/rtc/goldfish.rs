use rpsyscall::get_tid;

pub fn timestamp(base_addr: usize) -> u64 {
  const NSEC_PER_SEC: u64 = 1000000000;
  let low = unsafe { (base_addr as *mut u32).read() as u64 };
  let high = unsafe { ((base_addr + 4) as *mut u32).read() as u64 };
  ((high << 32) | low) / NSEC_PER_SEC
}

pub fn server(base_addr: usize, _irq_num: usize) {
  info!("server started t{}", get_tid());
  let base_addr = base_addr + rpabi::platform::USER_SPACE_DRIVER_MMIO_OFFSET;
  rpsyscall::server_register(rpabi::server::SERVER_RTC).unwrap();
  loop {
    let (client_tid, _msg) = rpsyscall::message::Message::receive().unwrap();
    let mut msg = rpsyscall::message::Message::default();
    msg.a = timestamp(base_addr) as usize;
    let _ = msg.send_to(client_tid);
  }
}
