use crate::span::Span;
use std::{fmt, rc::Rc};
use TermKind::*;

#[macro_export]
macro_rules! U {
    ($kind:expr) => {
        std::rc::Rc::new($crate::untyped::Term::new($kind))
    };
}

#[derive(Debug)]
pub struct Term {
    kind: TermKind,
    span: Span,
}

#[derive(Debug)]
pub enum TermKind {
    True,
    False,
    If {
        cond: Rc<Term>,
        then_branch: Rc<Term>,
        else_branch: Rc<Term>,
    },
    Var {
        idx: u32,
        len: u32,
    },
    Fun {
        name: String,
        ty: Rc<Ty>,
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

    pub fn eval(self: &Rc<Self>, ctx: &mut Context) -> Rc<Self> {
        self.eval_1(ctx).unwrap_or_else(|| self.clone())
    }

    fn eval_1(self: &Rc<Self>, ctx: &mut Context) -> Option<Rc<Self>> {
        match &self.kind {
            If {
                cond,
                then_branch,
                else_branch,
            } => match &cond.kind {
                True => Some(then_branch.clone()),
                False => Some(else_branch.clone()),
                _ => Some(Rc::new(Term {
                    kind: If {
                        cond: cond.eval_1(ctx)?,
                        then_branch: then_branch.clone(),
                        else_branch: else_branch.clone(),
                    },
                    span: self.span,
                })),
            },
            Call { callee, arg } => match &callee.kind {
                Fun { term, .. } if arg.is_val(ctx) => Some(arg.subst_top(term.clone())),
                _ if callee.is_val(ctx) => {
                    let arg = arg.eval_1(ctx)?;
                    Some(Rc::new(Term {
                        kind: Call {
                            callee: callee.clone(),
                            arg,
                        },
                        span: self.span,
                    }))
                }
                _ => {
                    let callee = callee.eval_1(ctx)?;
                    Some(Rc::new(Term {
                        kind: Call {
                            callee,
                            arg: arg.clone(),
                        },
                        span: self.span,
                    }))
                }
            },
            _ => None,
        }
    }

    pub fn subst_top(self: &Rc<Self>, subst_term: Rc<Self>) -> Rc<Self> {
        let subst_term = subst_term.shift(1);
        let term = self.subst(0, subst_term);
        term.shift(-1)
    }

    pub fn subst(self: &Rc<Self>, term_idx: u32, subst_term: Rc<Self>) -> Rc<Self> {
        self.map(0, &|span, ctx, idx, len| {
            if idx == term_idx + ctx {
                subst_term.shift(ctx as i32)
            } else {
                Rc::new(Term {
                    kind: Var { idx, len },
                    span,
                })
            }
        })
    }

    pub fn shift_above(self: &Rc<Self>, ctx: u32, dist: i32) -> Rc<Self> {
        self.map(ctx, &|span, ctx, idx, len| {
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

    pub fn shift(self: &Rc<Self>, dist: i32) -> Rc<Self> {
        self.shift_above(0, dist)
    }

    fn map<F>(self: &Rc<Self>, ctx: u32, map_fn: &F) -> Rc<Self>
    where
        F: Fn(Span, u32, u32, u32) -> Rc<Term>,
    {
        fn walk<F>(term: &Rc<Term>, ctx: u32, map_fn: &F) -> Rc<Term>
        where
            F: Fn(Span, u32, u32, u32) -> Rc<Term>,
        {
            let kind = match &term.kind {
                True | False => return term.clone(),
                If {
                    cond,
                    then_branch,
                    else_branch,
                } => If {
                    cond: walk(cond, ctx, map_fn),
                    then_branch: walk(then_branch, ctx, map_fn),
                    else_branch: walk(else_branch, ctx, map_fn),
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

        walk(self, ctx, map_fn)
    }

    pub fn print(&self, ctx: &mut Context, buf: &mut String) {
        match &self.kind {
            True => buf.push_str("true"),
            False => buf.push_str("false"),
            If {
                cond,
                then_branch,
                else_branch,
            } => {
                buf.push_str("if ");
                cond.print(ctx, buf);
                buf.push_str(" { ");
                then_branch.print(ctx, buf);
                buf.push_str(" } else { ");
                else_branch.print(ctx, buf);
                buf.push_str(" }");
            }
            Fun { name, term, .. } => {
                let x1 = ctx.pick_fresh_name(name);
                buf.push_str("(lambda ");
                buf.push_str(&x1);
                buf.push_str(". ");
                term.print(ctx, buf);
                buf.push(')');
            }
            Call { callee, arg } => {
                buf.push('(');
                callee.print(ctx, buf);
                buf.push(' ');
                arg.print(ctx, buf);
                buf.push(')');
            }
            Var { idx, len } => {
                if ctx.len() == *len as usize {
                    buf.push_str(ctx.index_to_name(*idx as usize));
                } else {
                    buf.push_str("[bad index]");
                }
            }
        }
    }

    pub fn type_of(&self, ctx: &Context) -> Rc<Ty> {
        match &self.kind {
            True | False => Rc::new(Ty::Bool),
            If {
                cond,
                then_branch,
                else_branch,
            } => {
                if cond.type_of(ctx) == Rc::new(Ty::Bool) {
                    let ty1 = then_branch.type_of(ctx);
                    let ty2 = else_branch.type_of(ctx);
                    if ty1 == ty2 {
                        return ty1;
                    } else {
                        quit!("Arms of Conditionals have different types");
                    }
                } else {
                    quit!("Guard of conditional must be a boolean");
                }
            }
            Var { idx, .. } => ctx.get_ty(*idx as usize),
            Fun { name, ty, term } => {
                let ctx = ctx.add_binding(name, Binding::Variable(ty.clone()));
                let to = term.type_of(&ctx);
                Rc::new(Ty::Arrow {
                    from: ty.clone(),
                    to,
                })
            }
            Call { callee, arg } => {
                let ty_callee = callee.type_of(ctx);
                let ty_arg = arg.type_of(ctx);
                match &*ty_callee {
                    Ty::Arrow { from, to } => {
                        if from == &ty_arg {
                            return to.clone();
                        } else {
                            quit!(
                                "Parameter type mismatch: expected: {:?}, actual: {:?}",
                                ty_arg,
                                from,
                            );
                        }
                    }
                    _ => quit!("Arrow type expected"),
                }
            }
        }
    }
}

#[derive(Default)]
pub struct Context {
    list: Vec<(String, Binding)>,
}

impl Context {
    pub fn len(&self) -> usize {
        self.list.len()
    }

    pub fn index_to_name(&self, index: usize) -> &str {
        &self.list[index].0
    }

    pub fn pick_fresh_name(&mut self, name: &str) -> String {
        let mut name = name.to_owned();
        if self.is_name_bound(&name) {
            name.push('\'');
            self.pick_fresh_name(&name)
        } else {
            self.list.push((name.clone(), Binding::Name));
            name
        }
    }

    pub fn is_name_bound(&self, name: &str) -> bool {
        self.list.iter().any(|(n, _)| n == name)
    }

    pub fn add_binding(&self, name: &str, binding: Binding) -> Self {
        let mut list = self.list.clone();
        list.push((name.to_owned(), binding));
        Self { list }
    }

    pub fn get_ty(&self, index: usize) -> Rc<Ty> {
        match self.get_binding(index) {
            Binding::Variable(ty) => ty.clone(),
            _ => quit!(
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
    Variable(Rc<Ty>),
}

#[derive(Clone, PartialEq)]
pub enum Ty {
    Bool,
    Arrow { from: Rc<Ty>, to: Rc<Ty> },
}

impl fmt::Debug for Ty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Ty::Bool => f.write_str("Bool"),
            Ty::Arrow { from, to } => write!(f, "{:?} -> {:?}", from, to),
        }
    }
}
