use crate::lib::thread::thread_destroy;

pub fn handle() {
  match crate::lib::cpu::cpu().running_thread() {
    None => { panic!("isr: default: no running thread") }
    Some(t) => {
      thread_destroy(t);
      crate::lib::cpu::cpu().schedule();
    }
  }
}
