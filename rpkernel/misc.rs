#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::kernel::print::print_arg(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ({
        $crate::kernel::print::print_arg(format_args!("{}\n", format_args!($($arg)*)));
    })
}

#[lang = "eh_personality"]
#[no_mangle]
pub extern fn rust_eh_personality() {
  error!("rust_eh_personality called");
  loop {}
}

#[cfg(not(target_arch = "x86_64"))]
#[allow(non_snake_case)]
#[no_mangle]
extern "C" fn _Unwind_Resume(arg: usize) -> ! {
  unwind::unwind_resume(arg)
}
