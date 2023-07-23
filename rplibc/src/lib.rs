#![no_std]
#![feature(lang_items)]

#[lang = "eh_personality"]
#[no_mangle]
pub extern fn rust_eh_personality() {
  loop {}
}

#[no_mangle]
extern "C" fn _Unwind_Resume(_arg: usize) -> ! {
  loop {}
}

extern "C" {
    fn main(arg: *const u8) -> i32;
}

#[no_mangle]
extern "C" fn _start(arg: *const u8) {
    rpstdlib::heap::init();
    unsafe {
        main(arg);
    }
    rpstdlib::exit();
}

#[no_mangle]
extern "C" fn putc(c: u8) {
    rpsyscall::putc(c as char);
}

#[no_mangle]
extern "C" fn getc() -> u8 {
    rpstdlib::stdio::getchar()
}

#[no_mangle]
extern "C" fn strlen(str: *const u8) -> usize {
    let mut i = 0;
    unsafe {
        loop {
            if str.add(i).read() == 0 {
                return i;
            }
            i += 1;
        }
    }
}
