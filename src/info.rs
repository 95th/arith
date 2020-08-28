#[derive(Debug, Copy, Clone)]
pub struct Info {
    pub lo: usize,
    pub hi: usize,
}

impl Info {
    pub const DUMMY: Info = Info { lo: 0, hi: 0 };
}
