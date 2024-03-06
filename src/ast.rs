use std::fmt;

use crate::token::Token;

#[derive(Debug)]
pub struct Unary {
    operator: Token,
    right: Box<Formula>,
}

#[derive(Debug)]
pub struct Binary {
    operator: Token,
    left: Box<Formula>,
    right: Box<Formula>,
}

#[derive(Debug)]
pub struct Leaf {
    ident: Token,
}

#[derive(Debug)]
pub enum Formula {
    Unary(Unary),
    Binary(Binary),
    Leaf(Leaf),
    Eof,
}

/* pub struct FormulaSet {
    f: Vec<Formula>,
} */

impl Formula {
    pub fn new_leaf(ident: Token) -> Self {
        Self::Leaf(Leaf { ident })
    }
    pub fn new_binary(left: Formula, operator: Token, right: Formula) -> Self {
        Self::Binary(Binary {
            left: Box::new(left),
            operator,
            right: Box::new(right),
        })
    }
    pub fn new_unary(operator: Token, right: Formula) -> Self {
        Self::Unary(Unary {
            operator,
            right: Box::new(right),
        })
    }
}

impl fmt::Display for Formula {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Formula::Leaf(l) => format!("{}", l.ident),
                Formula::Unary(u) => format!("({}{})", u.operator, u.right),
                Formula::Binary(b) => format!("({} {} {})", b.left, b.operator, b.right),
                Formula::Eof => format!("EOF"),
            }
        )
    }
}
