use crate::lib::cpu::CoreTrait;

pub fn interrupt() {
  crate::driver::timer::next();
  crate::lib::cpu::current().schedule();
}
