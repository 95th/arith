use crate::span::Span;

pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

#[derive(Clone, Copy)]
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
