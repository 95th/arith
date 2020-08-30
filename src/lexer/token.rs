use super::Symbol;
use crate::span::Span;

#[derive(Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
    pub symbol: Symbol,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TokenKind {
    // Keywords
    Lambda,
    True,
    False,

    // Other Identifier
    Ident,
    Number,

    OpenParen,
    CloseParen,
    OpenBrace,
    CloseBrace,
    Plus,
    Minus,
    Star,
    Slash,
    Comma,
    Dot,
    Semi,
    Colon,
    Not,
    Gt,
    Lt,
    Eq,

    Eof,
}
