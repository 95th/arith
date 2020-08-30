use crate::{
    ast::Expr,
    lexer::{Lexer, Symbol, Token, TokenKind},
    span::Span,
};
use std::rc::Rc;

pub struct Parser {
    lexer: Lexer,
    curr: Token,
    prev: Token,
    src: Rc<String>,
}

impl Parser {
    pub fn new(src: Rc<String>) -> Self {
        let mut lexer = Lexer::new(src.clone());
        let curr = lexer.next_token();
        Self {
            lexer,
            curr,
            prev: Token {
                kind: TokenKind::Eof,
                span: Span::dummy(),
                symbol: Symbol::dummy(),
            },
            src,
        }
    }

    pub fn parse_expr(&mut self) -> Expr {
        todo!()
    }
}
