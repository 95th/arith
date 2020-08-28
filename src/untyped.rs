use crate::info::Info;
use std::rc::Rc;
use TermKind::*;

#[derive(Clone)]
pub struct Term {
    kind: TermKind,
    info: Info,
}

#[derive(Clone)]
pub enum TermKind {
    Variable { idx: u32, len: u32 },
    Abstraction { name: String, term: Rc<Term> },
    Application { target: Rc<Term>, val: Rc<Term> },
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
            Abstraction { .. } => true,
            _ => false,
        }
    }

    pub fn eval(self: &Rc<Self>, ctx: &mut Context) -> Rc<Self> {
        self.eval_1(ctx).unwrap_or_else(|| self.clone())
    }

    fn eval_1(self: &Rc<Self>, ctx: &mut Context) -> Option<Rc<Self>> {
        match &self.kind {
            Application { target, val } => match &target.kind {
                Abstraction { term, .. } if val.is_val(ctx) => Some(val.subst_top(term.clone())),
                _ if target.is_val(ctx) => {
                    let val = val.eval_1(ctx)?;
                    Some(Rc::new(Term {
                        kind: Application {
                            target: target.clone(),
                            val,
                        },
                        info: self.info,
                    }))
                }
                _ => {
                    let target = target.eval_1(ctx)?;
                    Some(Rc::new(Term {
                        kind: Application {
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
                subst_term.clone().shift(ctx as i32)
            } else {
                Rc::new(Term {
                    kind: Variable { idx, len },
                    info,
                })
            }
        })
    }

    pub fn shift_above(self: &Rc<Self>, ctx: u32, dist: i32) -> Rc<Self> {
        self.map(ctx, &|info, ctx, idx, len| {
            let kind = Variable {
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
                Variable { idx, len } => return map_fn(term.info, ctx, *idx, *len),
                Abstraction { name, term } => Abstraction {
                    name: name.clone(),
                    term: walk(term, ctx + 1, map_fn),
                },
                Application { target, val } => Application {
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
            Abstraction { name, term } => {
                let x1 = ctx.pick_fresh_name(name);
                buf.push_str("(lambda ");
                buf.push_str(&x1);
                buf.push_str(". ");
                term.print(ctx, buf);
                buf.push_str(" )");
            }
            Application { target, val } => {
                buf.push('(');
                target.print(ctx, buf);
                buf.push(' ');
                val.print(ctx, buf);
                buf.push(')');
            }
            Variable { idx, len } => {
                if ctx.len() == *len as usize {
                    buf.push_str(ctx.index_to_name(*idx as usize));
                } else {
                    buf.push_str("[bad index]");
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
            self.list.push((name.clone(), Binding::NameBind));
            name
        }
    }

    pub fn is_name_bound(&self, name: &str) -> bool {
        self.list.iter().any(|(n, _)| n == name)
    }
}

pub enum Binding {
    NameBind,
}
