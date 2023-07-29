#![no_std]
#![no_main]
#![feature(format_args_nl)]
#![feature(alloc_error_handler)]

extern crate alloc;
#[macro_use]
extern crate rpstdlib;

use alloc::vec::Vec;
use rpstdlib::rtc::rtc_time64_to_tm;

#[no_mangle]
fn main(arg: Vec<&'static str>) -> i32 {
  if arg.len() == 0 {
    println!("usage: stat FILE...");
    return 0;
  }
  let path = arg[0];
  let file = rpstdlib::fs::File::open(path).expect("open file failed");

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
  0
}
