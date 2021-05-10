use crate::lib::cpu::CoreTrait;

pub fn handle() {
  match crate::lib::cpu::current().running_thread() {
    None => { panic!("isr: default: no running thread") }
    Some(t) => {
      t.destroy();
    }
  }
}
