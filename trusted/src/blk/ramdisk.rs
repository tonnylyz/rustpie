use core::intrinsics::volatile_store;
macro_rules! include_bytes_align_as {
    ($align_ty:ty, $path:literal) => {{
        #[repr(C)]
        pub struct AlignedAs<Align, Bytes: ?Sized> {
            pub _align: [Align; 0],
            pub bytes: Bytes,
        }

        static ALIGNED: &AlignedAs::<$align_ty, [u8]> = &AlignedAs {
            _align: [],
            bytes: *include_bytes!($path),
        };

        &ALIGNED.bytes
    }};
}

#[repr(align(4096))]
struct Align4096;
static RAMDISK: &'static [u8] = include_bytes_align_as!(Align4096, "../../../ramdisk.img");

pub fn server() {
  let ramdisk = unsafe { core::slice::from_raw_parts_mut(RAMDISK.as_ptr() as usize as *mut u8, RAMDISK.len()) };
  let ramdisk_addr = ramdisk.as_ptr() as usize;
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
      if msg.d == cs::blk::action::READ {
        // Operation::Read
        let buf = unsafe {
          core::slice::from_raw_parts_mut(buf as *mut u8, count * 512)
        };
        buf.copy_from_slice(&ramdisk[start..end]);
      } else {
        // Operation::Write
        let buf = unsafe {
          core::slice::from_raw_parts(buf as *const u8, count * 512)
        };
        // ramdisk[start..end].copy_from_slice(buf);
        for i in start..end {
          unsafe { volatile_store((ramdisk_addr + i) as *mut u8, buf[i - start]); }
        }
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
