use std::fmt;

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
            Kind::Invalid => write!(f, "INVALID",),
            Kind::Eof => write!(f, "EOF",),
            Kind::Identifier => write!(f, "IDENTIFIER",),
            Kind::ParenL => write!(f, "(",),
            Kind::ParenR => write!(f, ")",),
            Kind::And => write!(f, "&",),
            Kind::Or => write!(f, "|",),
            Kind::Not => write!(f, "~",),
            Kind::Implies => write!(f, "=>",),
            Kind::Equiv => write!(f, "<=>",),
            Kind::SemiColon => write!(f, ";",),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Token {
    kind: Kind,
    literal: String, // @perf would it be better to have an ID with and HashMap instead?
    row: usize,
    col: usize,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.literal)
    }
}

impl Token {
    pub fn new(kind: Kind, literal: String, row: usize, col: usize) -> Token {
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

