use std::env;
use std::path::Path;
use std::process::Command;

const RISCV_USER_IMAGE: &'static str = "user/riscv64.elf";
const AARCH64_USER_IMAGE: &'static str = "user/aarch64.elf";

fn main() {
  let target = env::var("TARGET").expect("TARGET was not set");
  let out_dir = env::var("OUT_DIR").unwrap();
  if target.contains("riscv64") {
    if !Path::new(RISCV_USER_IMAGE).exists() {
      println!("cargo:warning=USER_IMAGE_MISSING");
      panic!();
    }
    println!("cargo:rerun-if-changed={}", RISCV_USER_IMAGE);
    Command::new("ld.lld")
      .args(&["-m", "elf64lriscv", "-r", "-b", "binary", "-o"])
      .arg(&format!("{}/user_image.riscv64.o", out_dir))
      .arg(RISCV_USER_IMAGE)
      .status().unwrap();
    Command::new("llvm-ar")
      .arg("crus")
      .arg(&format!("{}/libuserspace.a", out_dir))
      .arg(&format!("{}/user_image.riscv64.o", out_dir))
      .status().unwrap();
  } else if target.contains("aarch64") {
    if !Path::new(AARCH64_USER_IMAGE).exists() {
      println!("cargo:warning=USER_IMAGE_MISSING");
      panic!();
    }
    println!("cargo:rerun-if-changed={}", AARCH64_USER_IMAGE);
    Command::new("ld.lld")
      .args(&["-m", "aarch64elf", "-r", "-b", "binary", "-o"])
      .arg(&format!("{}/user_image.aarch64.o", out_dir))
      .arg(AARCH64_USER_IMAGE)
      .status().unwrap();
    Command::new("llvm-ar")
      .arg("crus")
      .arg(&format!("{}/libuserspace.a", out_dir))
      .arg(&format!("{}/user_image.aarch64.o", out_dir))
      .status().unwrap();
  }
  println!("cargo:rustc-link-search=native={}", out_dir);
  println!("cargo:rustc-link-lib=static=userspace");
}
