#[derive(Eq, PartialEq)]
pub enum InterruptNo{
    Timer,
    Numbered(usize),
}

pub trait InterruptController {
    fn init(&self);

    fn enable(&self, int: InterruptNo);
    fn disable(&self, int: InterruptNo);

    fn fetch(&self) -> Option<InterruptNo>;
    fn finish(&self, int: InterruptNo);
}
