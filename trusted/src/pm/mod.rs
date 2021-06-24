use libtrusted::message::Message;
use libtrusted::mm::{Entry, EntryLike};
use microcall::{get_tid, get_asid, mem_map, mem_unmap};
use common::PAGE_SIZE;
use alloc::string::String;

fn process_request(va_tmp:usize, asid: u16, msg: &Message) -> Result<(), &'static str> {
  match msg.a {
    1 => { // SPAWN
      if msg.b % PAGE_SIZE != 0 {
        return Err("Alignment");
      }
      mem_map(asid, msg.b, 0, va_tmp, Entry::default().attribute()).map_err(|_| "Mem map Error")?;
      let buf = unsafe { core::slice::from_raw_parts(va_tmp as *const u8, PAGE_SIZE) };
      let mut length= 0;
      for i in 0..PAGE_SIZE {
        if buf[i] == 0 {
          length = i;
          break
        }
      }
      if length == 0 || length >= 128 {
        return Err("MalformedString");
      }
      let buf = unsafe { core::slice::from_raw_parts(va_tmp as *const u8, length) };
      let path = core::str::from_utf8(buf).map_err(|_| "MalEncoded")?;
      println!("[PM] ready to spawn {}", path);
      let asid = libtrusted::loader::spawn(path, msg.c).map_err(|e| {
        println!("[PM] spawn {}",e);
        "Spawn"
      })?;
      mem_unmap(0, va_tmp);
      Ok(())
    }
    _ => {
      Err("UnknownRequest")
    }
  }
}

pub fn server() {
  println!("[PM] server started t{}",  get_tid());
  microcall::server_register(common::server::SERVER_PM).unwrap();
  let va_tmp = libtrusted::mm::virtual_page_alloc(1);
  loop {
    let (tid, msg) = Message::receive();
    println!("[PM] t{}: {:x?}", tid, msg);
    let asid = get_asid(tid);
    match process_request(va_tmp, asid, &msg) {
      Ok(_) => {
        let mut msg = Message::default();
        msg.a = 0;
        msg.send_to(tid).unwrap();
      }
      Err(e) => {
        println!("[PM] ERR {}", e);
        let mut msg = Message::default();
        msg.a = 1;
        msg.send_to(tid).unwrap();
      }
    }
  }
}