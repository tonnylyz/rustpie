pub fn init() {
    // todo!()
    println!("TODO setup IDT");
}

core::arch::global_asm!(include_str!("interrupt.S"));