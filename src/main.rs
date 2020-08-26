use self::TermKind::*;
use typed_arena::Arena;

macro_rules! T {
    ($kind:expr, $arena:expr) => {
        $arena.alloc(Term::new($kind))
    };
}

fn main() {
    let arena = &Arena::new();
    let term = T![
        If(
            T![True, arena],
            T![Pred(T![Zero, arena]), arena],
            T![True, arena]
        ),
        arena
    ];
    println!("{:?}", term.clone().eval(arena).unwrap_or(term));
}

#[derive(Debug, Clone)]
pub struct Term<'a> {
    kind: TermKind<'a>,
    info: Info,
}

impl<'a> Term<'a> {
    pub fn new(kind: TermKind<'a>) -> Self {
        Self {
            kind,
            info: DUMMY_INFO,
        }
    }
}

#[derive(Debug, Clone)]
pub enum TermKind<'a> {
    True,
    False,
    Zero,
    If(&'a Term<'a>, &'a Term<'a>, &'a Term<'a>),
    Succ(&'a Term<'a>),
    Pred(&'a Term<'a>),
    IsZero(&'a Term<'a>),
}

#[derive(Debug, Copy, Clone)]
pub struct Info {
    pub lo: usize,
    pub hi: usize,
}

const DUMMY_INFO: Info = Info { lo: 0, hi: 0 };

impl<'a> Term<'a> {
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

    pub fn eval<'t>(&'t self, arena: &'t Arena<Term<'t>>) -> Option<&'t Term<'t>> {
        let t = match &self.kind {
            If(cond, t2, t3) => match &cond.kind {
                True => *t2,
                False => *t3,
                _ => arena.alloc(Term {
                    kind: If(cond.eval(arena)?, t2, t3),
                    info: self.info,
                }),
            },
            Succ(t) => arena.alloc(Term {
                kind: Succ(t.eval(arena)?),
                info: self.info,
            }),
            Pred(t) => match t.kind {
                Zero => T![Zero, arena],
                Succ(t) if t.is_numeric() => t,
                _ => arena.alloc(Term {
                    kind: Pred(t.eval(arena)?),
                    info: self.info,
                }),
            },
            IsZero(t) => match t.kind {
                Zero => T![True, arena],
                Succ(t) if t.is_numeric() => T![False, arena],
                _ => arena.alloc(Term {
                    kind: IsZero(t.eval(arena)?),
                    info: self.info,
                }),
            },
            _ => return None,
        };
        Some(t)
    }

    pub fn eval2<'t>(&'t self, arena: &'t Arena<Term<'t>>) -> Option<&'t Term<'t>> {
        let t = match &self.kind {
            If(cond, t2, t3) => match cond.eval2(arena)?.kind {
                True => *t2,
                False => t3,
                _ => return None,
            },
            Succ(t) => arena.alloc(Term {
                kind: Succ(t.eval2(arena)?),
                info: self.info,
            }),
            Pred(t) => match t.eval2(arena)?.kind {
                Zero => T![Zero, arena],
                Succ(t) if t.is_numeric() => t,
                _ => return None,
            },
            IsZero(t) => match t.eval2(arena)?.kind {
                Zero => T![True, arena],
                Succ(t) if t.is_numeric() => T![False, arena],
                _ => return None,
            },
            _ => return Some(self),
        };
        Some(t)
    }
}
