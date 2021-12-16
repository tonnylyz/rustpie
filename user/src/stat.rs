#![no_std]
#![no_main]
#![feature(format_args_nl)]
#![feature(alloc_error_handler)]

extern crate alloc;
#[macro_use]
extern crate exported;


use exported::rtc::rtc_time64_to_tm;
use fs::File;

#[no_mangle]
fn _start(arg: *const u8) {
  let arg = exported::parse(arg);
  if arg.len() == 0 {
    println!("usage: stat FILE...");
    exported::exit();
  }
  let path = arg[0];
  let file = File::open(path).expect("open file failed");

  match file.stat() {
    Ok(stat) => {
      println!(
"  File: {}
  Size: {}        	Blocks: {}
Device: {} 	Inode: {}   Links: {}
Access: {:o}  Uid: {}   Gid: {}
Access: {}
Modify: {}
Create: {}",
        path,
        stat.st_size,
        stat.st_blocks,
        stat.st_dev,
        stat.st_ino,
        stat.st_nlink,
        stat.st_mode,
        stat.st_uid,
        stat.st_gid,
        rtc_time64_to_tm(stat.st_atime),
        rtc_time64_to_tm(stat.st_mtime),
        rtc_time64_to_tm(stat.st_ctime),
      );
    }
    Err(e) => {
      println!("{}", e);
    }
  }
  exported::exit();
}
