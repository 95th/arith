use crate::{
    ast::Expr,
    lexer::{Lexer, Symbol, Token, TokenKind, TokenKind::*},
    span::Span,
};
use std::rc::Rc;

pub struct Parser {
    lexer: Lexer,
    curr: Token,
    prev: Token,
}

impl Parser {
    pub fn new(src: Rc<String>) -> Self {
        let mut lexer = Lexer::new(src.clone());
        let curr = lexer.next_token();
        Self {
            lexer,
            curr,
            prev: Token {
                kind: Eof,
                span: Span::dummy(),
                symbol: Symbol::dummy(),
            },
        }
    }

    pub fn parse_expr(&mut self) -> Expr {
        if self.eat(True) {
            Expr::True
        } else if self.eat(False) {
            Expr::False
        } else if self.eat(If) {
            let cond = self.parse_expr();
            self.consume(OpenBrace, "Expected '{' after If condition");
            let yes = self.parse_expr();
            self.consume(CloseBrace, "Expected '}'");
            self.consume(Else, "Expected 'else'");
            self.consume(OpenBrace, "Expected '{' after else");
            let no = self.parse_expr();
            self.consume(CloseBrace, "Expected '}'");
            Expr::If {
                cond: Box::new(cond),
                yes: Box::new(yes),
                no: Box::new(no),
            }
        } else {
            quit!("Unexpected token: {:?}", self.curr.kind);
        }
    }

    fn consume(&mut self, kind: TokenKind, msg: &str) {
        if self.eat(kind) {
            return;
        }

        quit!("{} at {:?}, expected: {:?}", msg, self.curr, kind);
    }

    fn eat(&mut self, kind: TokenKind) -> bool {
        if self.curr.kind == kind {
            self.advance();
            true
        } else {
            false
        }
    }

    fn advance(&mut self) {
        self.prev = std::mem::replace(&mut self.curr, self.lexer.next_token());
    }
}
