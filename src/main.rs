use self::TermKind::*;

macro_rules! T {
    ($kind:expr) => {
        Box::new(Term::new($kind))
    };
}

fn main() {
    let term = T![If(T![True], T![Pred(T![Zero])], T![True])];
    println!("{:?}", term.clone().eval().unwrap_or(term));
}

#[derive(Debug, Clone)]
pub struct Term {
    kind: TermKind,
    info: Info,
}

impl Term {
    pub fn new(kind: TermKind) -> Self {
        Self {
            kind,
            info: DUMMY_INFO,
        }
    }
}

#[derive(Debug, Clone)]
pub enum TermKind {
    True,
    False,
    Zero,
    If(Box<Term>, Box<Term>, Box<Term>),
    Succ(Box<Term>),
    Pred(Box<Term>),
    IsZero(Box<Term>),
}

#[derive(Debug, Copy, Clone)]
pub struct Info {
    pub lo: usize,
    pub hi: usize,
}

const DUMMY_INFO: Info = Info { lo: 0, hi: 0 };

impl Term {
    pub fn is_numeric(&self) -> bool {
        match &self.kind {
            Zero => true,
            Succ(t) => t.is_numeric(),
            _ => false,
        }
    }

    pub fn is_val(&self) -> bool {
        match &self.kind {
            True | False => true,
            _ if self.is_numeric() => true,
            _ => false,
        }
    }

    pub fn eval(self) -> Option<Box<Term>> {
        let t = match self.kind {
            If(cond, t2, t3) => match &cond.kind {
                True => t2,
                False => t3,
                _ => Box::new(Term {
                    kind: If(cond.eval()?, t2, t3),
                    info: self.info,
                }),
            },
            Succ(t) => Box::new(Term {
                kind: Succ(t.eval()?),
                info: self.info,
            }),
            Pred(t) => match t.kind {
                Zero => T![Zero],
                Succ(t) if t.is_numeric() => t,
                _ => Box::new(Term {
                    kind: Pred(t.eval()?),
                    info: self.info,
                }),
            },
            IsZero(t) => match t.kind {
                Zero => T![True],
                Succ(t) if t.is_numeric() => T![False],
                _ => Box::new(Term {
                    kind: IsZero(t.eval()?),
                    info: self.info,
                }),
            },
            _ => return None,
        };
        Some(t)
    }

    pub fn eval2(self) -> Option<Box<Term>> {
        let t = match self.kind {
            If(cond, t2, t3) => match cond.eval2()?.kind {
                True => t2,
                False => t3,
                _ => return None,
            },
            Succ(t) => Box::new(Term {
                kind: Succ(t.eval2()?),
                info: self.info,
            }),
            Pred(t) => match t.eval2()?.kind {
                Zero => T![Zero],
                Succ(t) if t.is_numeric() => t,
                _ => return None,
            },
            IsZero(t) => match t.eval2()?.kind {
                Zero => T![True],
                Succ(t) if t.is_numeric() => T![False],
                _ => return None,
            },
            _ => return Some(Box::new(self)),
        };
        Some(t)
    }
}
