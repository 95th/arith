use self::TermKind::*;
use crate::span::Span;
use std::rc::Rc;

#[macro_export]
macro_rules! T {
    ($kind:expr) => {
        Rc::new($crate::initial::Term::new($kind))
    };
}

#[derive(Debug, Clone)]
pub struct Term {
    kind: TermKind,
    span: Span,
}

impl Term {
    pub fn new(kind: TermKind) -> Self {
        Self {
            kind,
            span: Span::DUMMY,
        }
    }
}

#[derive(Debug, Clone)]
pub enum TermKind {
    True,
    False,
    Zero,
    If(Rc<Term>, Rc<Term>, Rc<Term>),
    Succ(Rc<Term>),
    Pred(Rc<Term>),
    IsZero(Rc<Term>),
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

    pub fn eval(self: &Rc<Self>) -> Rc<Self> {
        self.eval_1().unwrap_or_else(|| self.clone())
    }

    fn eval_1(self: &Rc<Self>) -> Option<Rc<Self>> {
        let t = match &self.kind {
            If(cond, t2, t3) => match &cond.kind {
                True => t2.clone(),
                False => t3.clone(),
                _ => Rc::new(Term {
                    kind: If(cond.eval_1()?, t2.clone(), t3.clone()),
                    span: self.span,
                }),
            },
            Succ(t) => Rc::new(Term {
                kind: Succ(t.eval_1()?),
                span: self.span,
            }),
            Pred(t) => match &t.kind {
                Zero => T![Zero],
                Succ(t) if t.is_numeric() => t.clone(),
                _ => Rc::new(Term {
                    kind: Pred(t.eval_1()?),
                    span: self.span,
                }),
            },
            IsZero(t) => match &t.kind {
                Zero => T![True],
                Succ(t) if t.is_numeric() => T![False],
                _ => Rc::new(Term {
                    kind: IsZero(t.eval_1()?),
                    span: self.span,
                }),
            },
            _ => return None,
        };
        Some(t)
    }
}
