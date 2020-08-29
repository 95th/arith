use crate::info::Info;
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
    kind: TermKind,
    info: Info,
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
    Abs {
        name: String,
        ty: Rc<Ty>,
        term: Rc<Term>,
    },
    App {
        target: Rc<Term>,
        val: Rc<Term>,
    },
}

impl Term {
    pub fn new(kind: TermKind) -> Self {
        Self {
            kind,
            info: Info::DUMMY,
        }
    }

    pub fn with_info(kind: TermKind, info: Info) -> Self {
        Self { kind, info }
    }

    pub fn is_val(&self, _ctx: &Context) -> bool {
        match &self.kind {
            Abs { .. } => true,
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
                    info: self.info,
                })),
            },
            App { target, val } => match &target.kind {
                Abs { term, .. } if val.is_val(ctx) => Some(val.subst_top(term.clone())),
                _ if target.is_val(ctx) => {
                    let val = val.eval_1(ctx)?;
                    Some(Rc::new(Term {
                        kind: App {
                            target: target.clone(),
                            val,
                        },
                        info: self.info,
                    }))
                }
                _ => {
                    let target = target.eval_1(ctx)?;
                    Some(Rc::new(Term {
                        kind: App {
                            target,
                            val: val.clone(),
                        },
                        info: self.info,
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
        self.map(0, &|info, ctx, idx, len| {
            if idx == term_idx + ctx {
                subst_term.shift(ctx as i32)
            } else {
                Rc::new(Term {
                    kind: Var { idx, len },
                    info,
                })
            }
        })
    }

    pub fn shift_above(self: &Rc<Self>, ctx: u32, dist: i32) -> Rc<Self> {
        self.map(ctx, &|info, ctx, idx, len| {
            let kind = Var {
                idx: if idx >= ctx {
                    ((idx as i32) + dist) as u32
                } else {
                    idx
                },
                len: ((len as i32) + dist) as u32,
            };
            Rc::new(Term { kind, info })
        })
    }

    pub fn shift(self: &Rc<Self>, dist: i32) -> Rc<Self> {
        self.shift_above(0, dist)
    }

    fn map<F>(self: &Rc<Self>, ctx: u32, map_fn: &F) -> Rc<Self>
    where
        F: Fn(Info, u32, u32, u32) -> Rc<Term>,
    {
        fn walk<F>(term: &Rc<Term>, ctx: u32, map_fn: &F) -> Rc<Term>
        where
            F: Fn(Info, u32, u32, u32) -> Rc<Term>,
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
                Var { idx, len } => return map_fn(term.info, ctx, *idx, *len),
                Abs { name, ty, term } => Abs {
                    name: name.clone(),
                    ty: ty.clone(),
                    term: walk(term, ctx + 1, map_fn),
                },
                App { target, val } => App {
                    target: walk(target, ctx, map_fn),
                    val: walk(val, ctx, map_fn),
                },
            };

            Rc::new(Term {
                kind,
                info: term.info,
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
            Abs { name, term, .. } => {
                let x1 = ctx.pick_fresh_name(name);
                buf.push_str("(lambda ");
                buf.push_str(&x1);
                buf.push_str(". ");
                term.print(ctx, buf);
                buf.push(')');
            }
            App { target, val } => {
                buf.push('(');
                target.print(ctx, buf);
                buf.push(' ');
                val.print(ctx, buf);
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
                        panic!("Arms of Conditionals have different types");
                    }
                } else {
                    panic!("Guard of conditional must be a boolean");
                }
            }
            Var { idx, .. } => ctx.get_ty(*idx as usize),
            Abs { name, ty, term } => {
                let ctx = ctx.add_binding(name, Binding::Variable(ty.clone()));
                let to = term.type_of(&ctx);
                Rc::new(Ty::Arrow {
                    from: ty.clone(),
                    to,
                })
            }
            App { target, val } => {
                let ty_target = target.type_of(ctx);
                let ty_val = val.type_of(ctx);
                match &*ty_target {
                    Ty::Arrow { from, to } => {
                        if from == &ty_val {
                            return to.clone();
                        } else {
                            panic!("Parameter type mismatch");
                        }
                    }
                    _ => panic!("Arrow type expected"),
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
            _ => panic!(
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

#[derive(Debug, Clone, PartialEq)]
pub enum Ty {
    Bool,
    Arrow { from: Rc<Ty>, to: Rc<Ty> },
}
