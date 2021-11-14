use libtrusted::mm::default_page_attribute;
use libtrusted::wrapper::request_wrapper;
use microcall::get_tid;
use microcall::message::Message;

#[inject::count_stmts]
#[inject::panic_inject]
#[inject::page_fault_inject]
fn mm(msg: Message, tid: usize) -> usize {
  let asid = microcall::get_asid(tid).unwrap();
  match msg.a {
    cs::mm::action::ALLOC => {
      match microcall::mem_alloc(asid, msg.b, default_page_attribute()) {
        Ok(_) => cs::mm::result::OK,
        Err(_) => cs::mm::result::ERR
      }
    }
    _ => {
      cs::mm::result::UNKNOWN_ACTION
    }
  }
}

pub fn server() {
  info!("server started t{}",  get_tid());
  microcall::server_register(common::server::SERVER_MM).unwrap();
  loop {
    let (client_tid, msg) = Message::receive().unwrap();
    let r = request_wrapper(mm, msg, client_tid).unwrap();
    let result = Message::new(r, 0, 0, 0);
    let _ = result.send_to(client_tid);
  }
}
