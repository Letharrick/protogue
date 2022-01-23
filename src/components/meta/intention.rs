#[derive(Debug)]
pub enum Intent {
    Walk,
    Grab,
    Throw,
    Attack,
}

pub struct Intention {
    pub intent: Intent,
}
