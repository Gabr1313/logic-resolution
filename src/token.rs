use std::{fmt, rc::Rc};

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Kind {
    Invalid,
    Eoi,
    Separator,
    Identifier,
    ParenL,
    ParenR,
    And,
    Or,
    Not,
    Implies,
    Equiv,
    Bang,
    Question,
    Number,
    Minus,
    Exit,
    Help,
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
            Kind::Invalid | Kind::Eoi => 1, // @todo remove
            _ => 2,
        }
    }

    // this is not automatically update with the lexer!
    pub fn as_str(&self) -> &str {
        match &self {
            Kind::Invalid => "INVALID",
            Kind::Eoi => "END OF INPUT",
            Kind::Separator => "SEPARATOR",
            Kind::Identifier => "IDENTIFIER",
            Kind::Number => "NUMBER",
            Kind::ParenL => "(",
            Kind::ParenR => ")",
            Kind::And => "&",
            Kind::Or => "|",
            Kind::Not => "~",
            Kind::Implies => "=>",
            Kind::Equiv => "<=>",
            Kind::Bang => "!",
            Kind::Question => "?",
            Kind::Minus => "-",
            Kind::Exit => "EXIT",
            Kind::Help => "HELP",
        }
    }

    pub fn is_sep(&self) -> bool {
        *self == Kind::Separator || *self == Kind::Eoi
    }
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Token {
    kind: Kind,
    literal: Rc<str>,
    row: usize,
    col: usize,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.literal)
    }
}

impl Token {
    pub fn new(kind: Kind, literal: Rc<str>, row: usize, col: usize) -> Token {
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
    pub fn literal(&self) -> Rc<str> {
        Rc::clone(&self.literal)
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
