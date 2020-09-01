use crate::{
    lexer::{Lexer, Symbol, Token, TokenKind, TokenKind::*},
    span::Span,
    untyped::{Term, TermKind},
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

    pub fn parse_expr(&mut self) -> Term {
        if self.eat(True) {
            Term {
                kind: TermKind::True,
                span: self.curr.span,
            }
        } else if self.eat(False) {
            Term {
                kind: TermKind::False,
                span: self.curr.span,
            }
        } else if self.eat(Zero) {
            Term {
                kind: TermKind::Zero,
                span: self.curr.span,
            }
        } else if self.eat(Succ) {
            let term = self.parse_expr();
            Term {
                kind: TermKind::Succ(Rc::new(term)),
                span: self.curr.span,
            }
        } else if self.eat(Pred) {
            let term = self.parse_expr();
            Term {
                kind: TermKind::Pred(Rc::new(term)),
                span: self.curr.span,
            }
        } else if self.eat(IsZero) {
            let term = self.parse_expr();
            Term {
                kind: TermKind::IsZero(Rc::new(term)),
                span: self.curr.span,
            }
        } else if self.eat(If) {
            let lo = self.curr.span;
            let cond = self.parse_expr();
            self.consume(OpenBrace, "Expected '{' after If condition");
            let yes = self.parse_expr();
            self.consume(CloseBrace, "Expected '}'");
            self.consume(Else, "Expected 'else'");
            self.consume(OpenBrace, "Expected '{' after else");
            let no = self.parse_expr();
            self.consume(CloseBrace, "Expected '}'");

            let span = lo.to(self.curr.span);
            Term {
                kind: TermKind::If {
                    cond: Rc::new(cond),
                    then_branch: Rc::new(yes),
                    else_branch: Rc::new(no),
                },
                span,
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
