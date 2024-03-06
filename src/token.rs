use std::{error::Error, fmt};

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Kind {
    Invalid,
    Eof,
    Ident,
    ParenL,
    ParenR,
    And,
    Or,
    Not,
    Implies,
    Equiv,
    SemiColon,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Token {
    pub kind: Kind,      // @todo: not pub
    pub literal: String, // @todo: not pub
    pub row: usize,      // @todo: not pub
    pub col: usize,      // @todo: not pub
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.literal)
    }
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
    pub fn precedence(&self) -> usize {
        // should be >= 1 because 0 is never taken
        match self.kind {
            Kind::Not => 9,
            Kind::And => 8,
            Kind::Or => 7,
            Kind::Implies => 6,
            Kind::Equiv => 5,
            Kind::SemiColon => 4,
            Kind::Ident => 3,
            Kind::ParenL | Kind::ParenR => 2,
            Kind::Invalid | Kind::Eof => 1,
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
