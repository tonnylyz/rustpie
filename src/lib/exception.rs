use crate::lib::thread::thread_destroy;

pub fn handle() {
  match crate::current_cpu().running_thread() {
    None => { panic!("isr: default: no running thread") }
    Some(t) => {
      thread_destroy(t);
      crate::current_cpu().schedule();
    }
  }
}
