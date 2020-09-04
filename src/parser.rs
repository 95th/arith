use crate::{
    lexer::{Lexer, Symbol, Token, TokenKind, TokenKind::*},
    span::Span,
    syntax::{Term, TermKind, TyContext},
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
                kind: Eof,
                span: Span::dummy(),
                symbol: Symbol::dummy(),
            },
            src,
        }
    }

    pub fn parse_expr(&mut self, tcx: &mut TyContext) -> Term {
        if self.eat(True) {
            Term {
                kind: TermKind::True,
                span: self.prev.span,
            }
        } else if self.eat(False) {
            Term {
                kind: TermKind::False,
                span: self.prev.span,
            }
        } else if self.eat(Zero) {
            Term {
                kind: TermKind::Zero,
                span: self.prev.span,
            }
        } else if self.eat(Succ) {
            let lo = self.prev.span;
            let term = self.parse_expr(tcx);
            Term {
                kind: TermKind::Succ(Rc::new(term)),
                span: lo.to(self.prev.span),
            }
        } else if self.eat(Pred) {
            let lo = self.prev.span;
            let term = self.parse_expr(tcx);
            Term {
                kind: TermKind::Pred(Rc::new(term)),
                span: lo.to(self.prev.span),
            }
        } else if self.eat(IsZero) {
            let lo = self.prev.span;
            let term = self.parse_expr(tcx);
            Term {
                kind: TermKind::IsZero(Rc::new(term)),
                span: lo.to(self.prev.span),
            }
        } else if self.eat(If) {
            let lo = self.prev.span;
            let cond = self.parse_expr(tcx);
            self.consume(OpenBrace, "Expected '{' after If condition");
            let yes = self.parse_expr(tcx);
            self.consume(CloseBrace, "Expected '}'");
            self.consume(Else, "Expected 'else'");
            self.consume(OpenBrace, "Expected '{' after else");
            let no = self.parse_expr(tcx);
            self.consume(CloseBrace, "Expected '}'");

            let span = lo.to(self.prev.span);
            Term {
                kind: TermKind::If {
                    cond: Rc::new(cond),
                    then_branch: Rc::new(yes),
                    else_branch: Rc::new(no),
                },
                span,
            }
        } else if self.eat(Pipe) {
            let lo = self.prev.span;
            self.consume(Ident, "Expected an indentifier for Lambda parameter");
            let name = self.prev.symbol;
            self.consume(Colon, "Expected ':' after lambda parameter");
            self.consume(Ident, "Expected type after lambda parameter");
            let ty_symbol = self.prev.symbol;

            let ty = ty_symbol.as_str_with(|s| match s {
                "Bool" => tcx.common.boolean,
                "Nat" => tcx.common.nat,
                x => quit!(&self.src, self.prev.span, "Unknown type: {}", x),
            });

            self.consume(Pipe, "Expected '|' after Lambda parameter");
            let body = self.parse_expr(tcx);

            let span = lo.to(self.prev.span);
            Term {
                kind: TermKind::Fun {
                    name,
                    ty,
                    term: Rc::new(body),
                },
                span,
            }
        } else {
            quit!(
                &self.src,
                self.curr.span,
                "Unexpected token: {:?}",
                self.curr.kind
            );
        }
    }

    fn consume(&mut self, kind: TokenKind, msg: &str) {
        if self.eat(kind) {
            return;
        }

        quit!(&self.src, self.curr.span, msg);
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
