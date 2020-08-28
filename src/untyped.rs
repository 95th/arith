use crate::info::Info;
use TermKind::*;

pub struct Term {
    kind: TermKind,
    info: Info,
}

pub enum TermKind {
    Variable { idx: u32, len: u32 },
    Abstraction { name: String, term: Box<Term> },
    Application { target: Box<Term>, val: Box<Term> },
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

    // pub fn subst_top(&self, subst_term: &Term) -> Box<Term> {
    //     self.subst(0, &subst_term.shift(1)).shift(todo!())
    // }

    pub fn subst(&self, term_idx: u32, subst_term: &Term) -> Box<Term> {
        self.map(0, &|info, ctx, idx, len| {
            if idx == term_idx + ctx {
                subst_term.shift(ctx)
            } else {
                Box::new(Term {
                    kind: Variable { idx, len },
                    info,
                })
            }
        })
    }

    pub fn shift_above(&self, ctx: u32, dist: u32) -> Box<Term> {
        self.map(ctx, &|info, ctx, idx, len| {
            let kind = Variable {
                idx: if idx >= ctx { idx + dist } else { idx },
                len: len + dist,
            };
            Box::new(Term { kind, info })
        })
    }

    pub fn shift(&self, dist: u32) -> Box<Term> {
        self.shift_above(0, dist)
    }

    fn map<F>(&self, ctx: u32, map_fn: &F) -> Box<Term>
    where
        F: Fn(Info, u32, u32, u32) -> Box<Term>,
    {
        fn walk<F>(term: &Term, ctx: u32, map_fn: &F) -> Box<Term>
        where
            F: Fn(Info, u32, u32, u32) -> Box<Term>,
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
            Box::new(Term {
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
