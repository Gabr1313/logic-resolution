use core::str;
use std::fmt;

pub type TokenKind = u8;

pub const INVALID: TokenKind = 0x00;
pub const EOF: TokenKind = 0x01;
pub const IDENT: TokenKind = 0x02;
pub const PAREN_L: TokenKind = 0x03;
pub const PAREN_R: TokenKind = 0x04;
pub const AND: TokenKind = 0x05;
pub const OR: TokenKind = 0x06;
pub const NOT: TokenKind = 0x07;
pub const IMPL: TokenKind = 0x08;
pub const EQUIV: TokenKind = 0x09;

pub fn token_to_string(t: TokenKind) -> &'static str {
    match t {
        INVALID => "INVALID",
        EOF => "EOF",
        IDENT => "IDENTIFIER",
        PAREN_L => "PAREN_L",
        PAREN_R => "PAREN_R",
        AND => "AND",
        OR => "OR",
        NOT => "NOT",
        IMPL => "IMPLIES",
        EQUIV => "EQUIV",
        _ => "UNKNOWN",
    }
}

#[derive(PartialEq)]
pub struct Token<'a> {
    pub kind: TokenKind,  // @todo: not pub
    pub literal: &'a str, // @todo: not pub
    pub row: usize,       // @todo: not pub
    pub col: usize,       // @todo: not pub
}

impl<'a> Token<'a> {
    pub fn new(kind: TokenKind, literal: &'a str, row: usize, col: usize) -> Self {
        Token {
            kind,
            literal,
            row,
            col,
        }
    }
}

impl<'a> fmt::Debug for Token<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Token")
            .field(
                "kind",
                &(format!("{}->{}", self.kind, token_to_string(self.kind))),
            )
            .field("literal", &self.literal)
            .field("row", &self.row)
            .field("col", &self.col)
            .finish()
    }
}
