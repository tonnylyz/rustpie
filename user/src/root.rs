use trusted::fs::{File, SeekFrom};

pub fn main(_arg: usize) {
  trusted::thread::spawn(|| {
    crate::blk::virtio_blk::server();
  });

  trusted::thread::spawn(|| {
    crate::fs::server();
  });

  trusted::thread::spawn(|| {
    println!("[TEST] client t{}", microcall::get_tid());
    let mut file = File::open("hello").ok().unwrap();
    let mut buf: [u8; 128] = [0; 128];
    let r = file.seek(SeekFrom::Start(0)).ok().unwrap();
    println!("[TEST] client seek {}", r);
    let r = file.read(&mut buf).ok().unwrap();
    println!("[TEST] client read {}", r);
    let str = core::str::from_utf8(&buf).unwrap();
    println!("[TEST] client str {}", str);
    microcall::thread_destroy(0);
  });

  loop {
    microcall::thread_yield();
  }
}
