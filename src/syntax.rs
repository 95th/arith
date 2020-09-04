use crate::{lexer::Symbol, span::Span};
use std::rc::Rc;
use TermKind::*;

#[macro_export]
macro_rules! U {
    ($kind:expr) => {
        std::rc::Rc::new($crate::untyped::Term::new($kind))
    };
}

#[derive(Debug)]
pub struct Term {
    pub kind: TermKind,
    pub span: Span,
}

#[derive(Debug)]
pub enum TermKind {
    True,
    False,
    Zero,
    If {
        cond: Rc<Term>,
        then_branch: Rc<Term>,
        else_branch: Rc<Term>,
    },
    Succ(Rc<Term>),
    Pred(Rc<Term>),
    IsZero(Rc<Term>),
    Var {
        idx: u32,
        len: u32,
    },
    Fun {
        name: Symbol,
        ty: TypeId,
        term: Rc<Term>,
    },
    Call {
        callee: Rc<Term>,
        arg: Rc<Term>,
    },
}

impl Term {
    pub fn new(kind: TermKind) -> Self {
        Self {
            kind,
            span: Span::dummy(),
        }
    }

    pub fn with_span(kind: TermKind, span: Span) -> Self {
        Self { kind, span }
    }

    pub fn is_val(&self, _ctx: &Context) -> bool {
        match &self.kind {
            Fun { .. } => true,
            _ => false,
        }
    }
}

pub struct Eval {
    src: Rc<String>,
}

impl Eval {
    pub fn new(src: Rc<String>) -> Self {
        Self { src }
    }

    pub fn eval(&self, term: &Rc<Term>, ctx: &mut Context) -> Rc<Term> {
        self.eval_1(term, ctx).unwrap_or_else(|| term.clone())
    }

    fn eval_1(&self, term: &Rc<Term>, ctx: &mut Context) -> Option<Rc<Term>> {
        match &term.kind {
            If {
                cond,
                then_branch,
                else_branch,
            } => {
                let cond = self.eval(cond, ctx);
                match &cond.kind {
                    True => Some(self.eval(then_branch, ctx)),
                    False => Some(self.eval(else_branch, ctx)),
                    _ => None,
                }
            }
            Call { callee, arg } => match &callee.kind {
                Fun { term, .. } if arg.is_val(ctx) => Some(self.subst_top(arg, term.clone())),
                _ if callee.is_val(ctx) => {
                    let arg = self.eval(arg, ctx);
                    Some(Rc::new(Term {
                        kind: Call {
                            callee: callee.clone(),
                            arg,
                        },
                        span: term.span,
                    }))
                }
                _ => {
                    let callee = self.eval(callee, ctx);
                    Some(Rc::new(Term {
                        kind: Call {
                            callee,
                            arg: arg.clone(),
                        },
                        span: term.span,
                    }))
                }
            },
            Succ(t) => Some(Rc::new(Term {
                kind: Succ(self.eval(t, ctx)),
                span: term.span,
            })),
            Pred(t) => {
                let t = self.eval(t, ctx);
                match &t.kind {
                    Zero => Some(Rc::new(Term {
                        kind: Zero,
                        span: term.span,
                    })),
                    Succ(t) => Some(t.clone()),
                    _ => return None,
                }
            }
            IsZero(t) => {
                let t = self.eval(t, ctx);
                let kind = match &t.kind {
                    Zero => True,
                    _ => False,
                };
                Some(Rc::new(Term {
                    kind,
                    span: term.span,
                }))
            }
            _ => None,
        }
    }

    pub fn subst_top(&self, term: &Rc<Term>, subst_term: Rc<Term>) -> Rc<Term> {
        let subst_term = self.shift(&subst_term, 1);
        let term = self.subst(term, 0, subst_term);
        self.shift(&term, -1)
    }

    pub fn subst(&self, term: &Rc<Term>, term_idx: u32, subst_term: Rc<Term>) -> Rc<Term> {
        self.map(term, 0, &|span, ctx, idx, len| {
            if idx == term_idx + ctx {
                self.shift(&subst_term, ctx as i32)
            } else {
                Rc::new(Term {
                    kind: Var { idx, len },
                    span,
                })
            }
        })
    }

    pub fn shift_above(&self, term: &Rc<Term>, ctx: u32, dist: i32) -> Rc<Term> {
        self.map(term, ctx, &|span, ctx, idx, len| {
            let kind = Var {
                idx: if idx >= ctx {
                    ((idx as i32) + dist) as u32
                } else {
                    idx
                },
                len: ((len as i32) + dist) as u32,
            };
            Rc::new(Term { kind, span })
        })
    }

    pub fn shift(&self, term: &Rc<Term>, dist: i32) -> Rc<Term> {
        self.shift_above(term, 0, dist)
    }

    fn map<F>(&self, term: &Rc<Term>, ctx: u32, map_fn: &F) -> Rc<Term>
    where
        F: Fn(Span, u32, u32, u32) -> Rc<Term>,
    {
        fn walk<F>(term: &Rc<Term>, ctx: u32, map_fn: &F) -> Rc<Term>
        where
            F: Fn(Span, u32, u32, u32) -> Rc<Term>,
        {
            let kind = match &term.kind {
                True | False | Zero => return term.clone(),
                If {
                    cond,
                    then_branch,
                    else_branch,
                } => If {
                    cond: walk(cond, ctx, map_fn),
                    then_branch: walk(then_branch, ctx, map_fn),
                    else_branch: walk(else_branch, ctx, map_fn),
                },
                Succ(t) => Succ(walk(t, ctx, map_fn)),
                Pred(t) => match &t.kind {
                    Zero => Zero,
                    Succ(t1) => return t1.clone(),
                    _ => Pred(walk(t, ctx, map_fn)),
                },
                IsZero(t) => match &t.kind {
                    Zero => True,
                    Succ(_) => False,
                    _ => IsZero(walk(t, ctx, map_fn)),
                },
                Var { idx, len } => return map_fn(term.span, ctx, *idx, *len),
                Fun { name, ty, term } => Fun {
                    name: name.clone(),
                    ty: ty.clone(),
                    term: walk(term, ctx + 1, map_fn),
                },
                Call { callee, arg } => Call {
                    callee: walk(callee, ctx, map_fn),
                    arg: walk(arg, ctx, map_fn),
                },
            };

            Rc::new(Term {
                kind,
                span: term.span,
            })
        }

        walk(term, ctx, map_fn)
    }

    pub fn print(&self, term: &Term, ctx: &mut Context, buf: &mut String) {
        match &term.kind {
            True => buf.push_str("true"),
            False => buf.push_str("false"),
            Zero => buf.push('0'),
            If {
                cond,
                then_branch,
                else_branch,
            } => {
                buf.push_str("if ");
                self.print(cond, ctx, buf);
                buf.push_str(" { ");
                self.print(then_branch, ctx, buf);
                buf.push_str(" } else { ");
                self.print(else_branch, ctx, buf);
                buf.push_str(" }");
            }
            Succ(t) => {
                buf.push_str("succ ");
                self.print(t, ctx, buf);
            }
            Pred(t) => {
                buf.push_str("pred ");
                self.print(t, ctx, buf);
            }
            IsZero(t) => {
                buf.push_str("iszero ");
                self.print(t, ctx, buf);
            }
            Fun { name, term, .. } => {
                let x1 = ctx.pick_fresh_name(*name);
                buf.push_str("(|");
                x1.as_str_with(|s| buf.push_str(s));
                buf.push_str("| ");
                self.print(term, ctx, buf);
                buf.push(')');
            }
            Call { callee, arg } => {
                buf.push('(');
                self.print(callee, ctx, buf);
                buf.push(' ');
                self.print(arg, ctx, buf);
                buf.push(')');
            }
            Var { idx, len } => {
                if ctx.len() == *len as usize {
                    let name = ctx.index_to_name(*idx as usize);
                    name.as_str_with(|s| buf.push_str(s));
                } else {
                    buf.push_str("[bad index]");
                }
            }
        }
    }

    pub fn type_of(&self, term: &Term, ctx: &Context, tyctx: &mut TyContext) -> TypeId {
        match &term.kind {
            True | False => tyctx.common.boolean,
            Zero => tyctx.common.nat,
            If {
                cond,
                then_branch,
                else_branch,
            } => {
                if self.type_of(cond, ctx, tyctx) == tyctx.common.boolean {
                    let ty1 = self.type_of(then_branch, ctx, tyctx);
                    let ty2 = self.type_of(else_branch, ctx, tyctx);
                    if ty1 == ty2 {
                        return ty1;
                    } else {
                        quit!(
                            &self.src,
                            term.span,
                            "Arms of Conditionals have different types",
                        );
                    }
                } else {
                    quit!(
                        &self.src,
                        cond.span,
                        "Guard of conditional must be a boolean"
                    );
                }
            }
            Succ(t) | Pred(t) => {
                let ty = self.type_of(t, ctx, tyctx);
                if ty == tyctx.common.nat {
                    return ty;
                } else {
                    quit!(&self.src, t.span, "argument must be a Nat");
                }
            }
            IsZero(t) => {
                if self.type_of(t, ctx, tyctx) == tyctx.common.nat {
                    return tyctx.common.boolean;
                } else {
                    quit!(&self.src, t.span, "argument must be a Nat");
                }
            }
            Var { idx, .. } => ctx.get_ty(&self.src, term.span, *idx as usize),
            Fun { name, ty, term } => {
                let ctx = ctx.add_binding(*name, Binding::Variable(*ty));
                let to = self.type_of(term, &ctx, tyctx);
                tyctx.new_arrow(*ty, to)
            }
            Call { callee, arg } => {
                let ty_callee = self.type_of(callee, ctx, tyctx);
                let ty_arg = self.type_of(arg, ctx, tyctx);
                match tyctx.get(ty_callee) {
                    &Ty::Arrow { from, to } => {
                        if from == ty_arg {
                            return to;
                        } else {
                            quit!(
                                &self.src,
                                term.span,
                                "Parameter type mismatch: expected: {:?}, actual: {:?}",
                                ty_arg,
                                from,
                            );
                        }
                    }
                    _ => quit!(&self.src, term.span, "Arrow type expected"),
                }
            }
        }
    }
}

#[derive(Default)]
pub struct Context {
    list: Vec<(Symbol, Binding)>,
}

impl Context {
    pub fn len(&self) -> usize {
        self.list.len()
    }

    pub fn index_to_name(&self, index: usize) -> Symbol {
        self.list[index].0
    }

    pub fn pick_fresh_name(&mut self, mut name: Symbol) -> Symbol {
        if self.is_name_bound(&name) {
            let mut buf = name.as_str_with(|s| s.to_owned());
            while self.is_name_bound(&name) {
                buf.push('\'');
                name = Symbol::intern(&buf);
            }
        }
        self.list.push((name, Binding::Name));
        name
    }

    pub fn is_name_bound(&self, name: &Symbol) -> bool {
        self.list.iter().any(|(n, _)| n == name)
    }

    pub fn add_binding(&self, name: Symbol, binding: Binding) -> Self {
        let mut list = self.list.clone();
        list.push((name, binding));
        Self { list }
    }

    pub fn get_ty(&self, src: &Rc<String>, span: Span, index: usize) -> TypeId {
        match self.get_binding(index) {
            Binding::Variable(ty) => *ty,
            _ => quit!(
                src,
                span,
                "Wrong kind of binding for variable: {}",
                self.index_to_name(index)
            ),
        }
    }

    pub fn get_binding(&self, index: usize) -> &Binding {
        &self.list[index].1
    }
}

#[derive(Debug, Clone)]
pub enum Binding {
    Name,
    Variable(TypeId),
}

pub struct TyContext {
    pub types: Vec<Ty>,
    pub common: CommonTypes,
}

impl TyContext {
    pub fn new() -> Self {
        let common = CommonTypes { boolean: 0, nat: 1 };
        let types = vec![Ty::Bool, Ty::Nat];
        Self { types, common }
    }

    pub fn new_arrow(&mut self, from: TypeId, to: TypeId) -> TypeId {
        let i = self.types.len();
        self.types.push(Ty::Arrow { from, to });
        i
    }

    pub fn new_ty(&mut self, symbol: Symbol) -> TypeId {
        symbol.as_str_with(|s| match s {
            "bool" => self.common.boolean,
            "nat" => self.common.nat,
            _ => todo!("Dont know how"),
        })
    }

    pub fn get(&self, id: TypeId) -> &Ty {
        &self.types[id]
    }

    pub fn print(&self, id: TypeId) {
        self.get(id).print(self);
        println!()
    }
}

pub struct CommonTypes {
    pub boolean: TypeId,
    pub nat: TypeId,
}

type TypeId = usize;

#[derive(Clone, PartialEq)]
pub enum Ty {
    Bool,
    Nat,
    Arrow { from: TypeId, to: TypeId },
}

impl Ty {
    fn print(&self, ctx: &TyContext) {
        match self {
            Ty::Bool => print!("Bool"),
            Ty::Nat => print!("Nat"),
            &Ty::Arrow { from, to } => {
                print!("|");
                ctx.get(from).print(ctx);
                print!("| ");
                ctx.get(to).print(ctx);
            }
        }
    }
}
