use std::process::Command;
use std::env;

fn main() {
  let target = env::var("TARGET").expect("TARGET was not set");
  let out_dir = env::var("OUT_DIR").unwrap();
  if target.contains("riscv64") {
    Command::new("ld.lld")
      .args(&["-m", "elf64lriscv", "-r", "-b", "binary", "-o"])
      .arg(&format!("{}/user_image.riscv64.o", out_dir))
      .arg("user/riscv64.elf")
      .status().unwrap();
    Command::new("rust-ar")
      .arg("crus")
      .arg(&format!("{}/libuserspace.a", out_dir))
      .arg(&format!("{}/user_image.riscv64.o", out_dir))
      .status().unwrap();
  } else if target.contains("aarch64") {
    Command::new("ld.lld")
      .args(&["-m", "aarch64elf", "-r", "-b", "binary", "-o"])
      .arg(&format!("{}/user_image.aarch64.o", out_dir))
      .arg("user/aarch64.elf")
      .status().unwrap();
    Command::new("rust-ar")
      .arg("crus")
      .arg(&format!("{}/libuserspace.a", out_dir))
      .arg(&format!("{}/user_image.aarch64.o", out_dir))
      .status().unwrap();
  }
  println!("cargo:rustc-link-search=native={}", out_dir);
  println!("cargo:rustc-link-lib=static=userspace");
}
