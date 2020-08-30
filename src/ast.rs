use crate::lexer::Symbol;

pub struct Lambda {
    pub arg: Symbol,
    pub body: Expr,
}

#[derive(Debug)]
pub enum Expr {
    True,
    False,
    If {
        cond: Box<Expr>,
        yes: Box<Expr>,
        no: Box<Expr>,
    },
}
