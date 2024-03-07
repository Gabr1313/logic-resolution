use std::{error::Error, fmt};

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Kind {
    Invalid,
    Eof,
    Identifier,
    ParenL,
    ParenR,
    And,
    Or,
    Not,
    Implies,
    Equiv,
    SemiColon,
}

impl Kind {
    pub fn precedence(&self) -> usize {
        // should be >= 1 because 0 is never read by the parser (default value)
        match self {
            Kind::Not => 7,
            Kind::And => 6,
            Kind::Or => 5,
            Kind::Implies => 4,
            Kind::Equiv => 3,
            Kind::SemiColon | Kind::Identifier | Kind::ParenL | Kind::ParenR => 2,
            Kind::Invalid | Kind::Eof => 1,
        }
    }
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            Self::Invalid => write!(f, "INVALID",),
            Self::Eof => write!(f, "EOF",),
            Self::Identifier => write!(f, "IDENTIFIER",),
            Self::ParenL => write!(f, "(",),
            Self::ParenR => write!(f, ")",),
            Self::And => write!(f, "&",),
            Self::Or => write!(f, "|",),
            Self::Not => write!(f, "!",),
            Self::Implies => write!(f, "=>",),
            Self::Equiv => write!(f, "<=>",),
            Self::SemiColon => write!(f, ";",),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Token {
    kind: Kind,
    literal: String, // @think: would it be better to have an ID with and HashMap instead?
    row: usize,
    col: usize,
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
    pub fn kind(&self) -> Kind {
        self.kind
    }
    pub fn literal(&self) -> &str {
        &self.literal
    }
    pub fn destory(self) -> String {
        self.literal
    }
    pub fn row(&self) -> usize {
        self.row
    }
    pub fn col(&self) -> usize {
        self.col
    }
    pub fn precedence(&self) -> usize {
        self.kind.precedence()
    }
}

#[derive(Debug)]
pub struct InvalidTokenErr {
    tok: String,
    row: usize,
    col: usize,
}

impl Error for InvalidTokenErr {}

impl InvalidTokenErr {
    pub fn new(tok: String, row: usize, col: usize) -> Box<Self> {
        Box::new(InvalidTokenErr { tok, col, row })
    }
}

impl fmt::Display for InvalidTokenErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "invalid token [{}:{}]: {}", self.row, self.col, self.tok)
    }
}
