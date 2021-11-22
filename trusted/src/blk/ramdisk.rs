
pub fn server() {
  let ramdisk = include_bytes!("../../../ramdisk.img");
  let ramdisk_mut = unsafe { core::slice::from_raw_parts_mut(ramdisk.as_ptr() as usize as *mut u8, ramdisk.len()) };
  info!("server started t{}",  microcall::get_tid());
  microcall::server_register(common::server::SERVER_BLK).unwrap();

  loop {
    let (client_tid, msg) = microcall::message::Message::receive().unwrap();
    if msg.d == cs::blk::action::READ || msg.d == cs::blk::action::WRITE {
      let sector = msg.a;
      let count = msg.b;
      let buf = msg.c;

      let start = sector * 512;
      let end = (sector + count) * 512;
      if msg.d == 0 {
        // Operation::Read
        let buf = unsafe {
          core::slice::from_raw_parts_mut(buf as *mut u8, count * 512)
        };
        buf.copy_from_slice(&ramdisk_mut[start..end]);
      } else {
        // Operation::Write
        let buf = unsafe {
          core::slice::from_raw_parts(buf as *mut u8, count * 512)
        };
        ramdisk_mut[start..end].copy_from_slice(buf);
      }

      let msg = microcall::message::Message::default();
      let _ = msg.send_to(client_tid);
    } else if msg.d == cs::blk::action::SIZE {
      let mut msg = microcall::message::Message::default();
      msg.a = ramdisk.len();
      let _ = msg.send_to(client_tid);
    } else {
      let mut msg = microcall::message::Message::default();
      msg.a = cs::blk::result::ERR;
      let _ = msg.send_to(client_tid);
    }
  }
}
