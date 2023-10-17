use rpsyscall::get_tid;

pub fn timestamp(base_addr: usize) -> u64 {
  unsafe { (base_addr as *mut u32).read() as u64 }
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
