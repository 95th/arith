use std::fmt;

#[derive(Copy, Clone)]
pub struct Span {
    pub lo: usize,
    pub hi: usize,
    pub line: usize,
}

impl Span {
    pub const fn dummy() -> Self {
        Self {
            lo: 0,
            hi: 0,
            line: 0,
        }
    }

    pub fn to(&self, other: Self) -> Self {
        Self {
            lo: self.lo.min(other.lo),
            hi: self.hi.max(other.hi),
            line: self.line.min(other.line),
        }
    }
}

impl fmt::Debug for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}..{}", self.lo, self.hi)
    }
}
