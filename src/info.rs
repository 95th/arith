use std::fmt;

#[derive(Copy, Clone)]
pub struct Info {
    pub lo: usize,
    pub hi: usize,
}

impl Info {
    pub const DUMMY: Info = Info { lo: 0, hi: 0 };
}

impl fmt::Debug for Info {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}..{}", self.lo, self.hi)
    }
}
