use self::TermKind::*;
use crate::info::Info;

#[macro_export]
macro_rules! T {
    ($kind:expr) => {
        Box::new($crate::initial::Term::new($kind))
    };
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
            info: Info::DUMMY,
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

    pub fn eval(self: Box<Self>) -> Result<Box<Term>, Box<Term>> {
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
            _ => return Err(self),
        };
        Ok(t)
    }
}
