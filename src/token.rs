use std::{error::Error, fmt};

#[derive(Debug, PartialEq, Eq)]
pub enum Kind {
    Invalid,
    Eof,
    Ident,
    ParenL,
    ParenR,
    And,
    Or,
    Not,
    Impl,
    Equiv,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Token {
    pub kind: Kind, // @todo: not pub
    pub literal: String, // @todo: not pub
    pub row: usize,      // @todo: not pub
    pub col: usize,      // @todo: not pub
}

impl Token {
    pub fn new(kind: Kind, literal: String, row: usize, col: usize) -> Self {
        Token {
            kind,
            literal,
            row,
            col,
        }
    }
}

#[derive(Debug)]
pub struct InvalidTokenErr {
    tok: String,
    row: usize,
    col: usize,
}

impl<'a> Error for InvalidTokenErr {}

impl<'a> InvalidTokenErr {
    pub fn new(tok: String, row: usize, col: usize) -> Box<Self> {
        Box::new(InvalidTokenErr { tok, col, row })
    }
}

impl fmt::Display for InvalidTokenErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "invalid token [{}:{}]: {}", self.row, self.col, self.tok)
    }
}
