#![no_std]
#![no_main]
#![feature(format_args_nl)]
#![feature(alloc_error_handler)]

extern crate alloc;
#[macro_use]
extern crate exported;
extern crate rlibc;

#[no_mangle]
fn _start(arg: *const u8) {
  let arg = exported::parse(arg);
  if arg.len() != 2 {
    println!("usage: write FILE TEXT...");
    exported::exit();
  }
  let path = arg[0];
  let file = fs::File::create(path);
  match file {
    Ok(mut file) => {
      file.write(arg[1].as_bytes()).expect("write file failed");
      file.write(&[b'\n']).expect("write file failed");
    }
    Err(e) => {
      println!("{}", e);
    }
  }
  exported::exit();
}
