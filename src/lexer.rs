mod symbol;
mod token;

use crate::span::Span;
use std::collections::HashMap;
use std::rc::Rc;
pub use symbol::Symbol;
use token::{Token, TokenKind, TokenKind::*};

lazy_static! {
    static ref KEYWORDS: HashMap<Symbol, TokenKind> = {
        let mut map = HashMap::new();
        map.insert(Symbol::intern("lambda"), TokenKind::Lambda);
        map.insert(Symbol::intern("true"), TokenKind::True);
        map.insert(Symbol::intern("false"), TokenKind::False);
        map
    };
}

pub struct Lexer {
    src: Rc<String>,
    start: usize,
    pos: usize,
    line: usize,
}

impl Lexer {
    pub fn new(src: &Rc<String>) -> Self {
        Self {
            src: src.clone(),
            start: 0,
            pos: 0,
            line: 1,
        }
    }

    pub fn next_token(&mut self) -> Token {
        while !self.eof() {
            self.start = self.pos;
            let kind = match self.next_char() {
                b'(' => OpenParen,
                b')' => CloseParen,
                b'{' => OpenBrace,
                b'}' => CloseBrace,
                b'+' => Plus,
                b'-' => Minus,
                b'*' => Star,
                b'/' => Slash,
                b',' => Comma,
                b'.' => Dot,
                b';' => Semi,
                b':' => Colon,
                b'!' => Not,
                b'=' => Eq,
                b'>' => Gt,
                b'<' => Lt,
                b'\n' => {
                    self.line += 1;
                    continue;
                }
                c if c.is_ascii_whitespace() => continue,
                c if c.is_ascii_alphabetic() => self.ident(),
                c if c.is_ascii_digit() => self.number(),
                c => quit!("Unknown character: {}", c as char),
            };
            return self.token(kind);
        }

        self.token(Eof)
    }

    fn ident(&mut self) -> TokenKind {
        self.eat_while(|c| c.is_ascii_alphanumeric() || c == b'_');
        let symbol = self.symbol();
        KEYWORDS
            .get(&symbol)
            .copied()
            .unwrap_or_else(|| TokenKind::Ident)
    }

    fn number(&mut self) -> TokenKind {
        self.eat_while(|c| c.is_ascii_digit());
        TokenKind::Number
    }

    fn token(&self, kind: TokenKind) -> Token {
        let span = self.span();
        Token { kind, span }
    }

    fn symbol(&self) -> Symbol {
        let s = &self.src[self.start..self.pos];
        Symbol::intern(s)
    }

    fn span(&self) -> Span {
        Span {
            lo: self.start,
            hi: self.pos,
            line: self.line,
        }
    }

    fn eat_while(&mut self, f: impl Fn(u8) -> bool) {
        while f(self.peek_char()) {
            self.advance();
        }
    }

    fn peek_char(&self) -> u8 {
        self.src
            .as_bytes()
            .get(self.pos)
            .copied()
            .unwrap_or_default()
    }

    fn next_char(&mut self) -> u8 {
        let c = self.peek_char();
        self.advance();
        c
    }

    fn advance(&mut self) {
        if !self.eof() {
            self.pos += 1;
        }
    }

    fn eof(&self) -> bool {
        self.pos >= self.src.len()
    }
}
