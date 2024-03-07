use crate::{token, Res};
use std::{error::Error, fmt};

#[derive(Debug, Clone)]
pub struct Unary {
    operator: token::Kind,
    right: Box<Formula>,
}

impl Unary {
    pub fn destroy(self) -> (token::Kind, Formula) {
        (self.operator, *self.right)
    }
}

#[derive(Debug, Clone)]
pub struct Binary {
    operator: token::Kind,
    left: Box<Formula>,
    right: Box<Formula>,
}

impl Binary {
    pub fn destroy(self) -> (Formula, token::Kind, Formula) {
        (*self.left, self.operator, *self.right)
    }
}

#[derive(Debug, Clone)]
pub struct Leaf {
    ident: String,
}

impl Leaf {
    pub fn destroy(self) -> String {
        self.ident
    }
}

#[derive(Debug, Clone)]
pub enum Formula {
    Unary(Unary),
    Binary(Binary),
    Leaf(Leaf),
    Eof,
}

impl fmt::Display for Formula {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Formula::Leaf(l) => l.ident.to_string(),
                Formula::Unary(u) => format!("({}{})", u.operator, u.right),
                Formula::Binary(b) => format!("({} {} {})", b.left, b.operator, b.right),
                Formula::Eof => "EOF".to_string(),
            }
        )
    }
}

impl Formula {
    pub fn new_leaf(ident: String) -> Self {
        Self::Leaf(Leaf { ident })
    }
    pub fn new_binary(left: Formula, operator: token::Kind, right: Formula) -> Self {
        Self::Binary(Binary {
            left: Box::new(left),
            operator,
            right: Box::new(right),
        })
    }
    pub fn new_unary(operator: token::Kind, right: Formula) -> Self {
        Self::Unary(Unary {
            operator,
            right: Box::new(right),
        })
    }
    fn is_and(&self) -> bool {
        match self {
            Formula::Binary(x) => x.operator == token::Kind::And,
            _ => false,
        }
    }

    // TODO: find a better name for this function
    /// push `!` inside, and substitue `=>` and`<=>`
    fn pre_distribute(self) -> Res<Self> {
        Ok(match self {
            Formula::Unary(x) => {
                let (operator, right) = x.destroy();
                match operator {
                    token::Kind::Not => right.negate_and_pre_distribute()?,
                    _ => {
                        return Err(InternalErrorTok::new(
                            operator,
                            "not a valid unary operator".to_string(),
                        ))
                    }
                }
            }
            Formula::Binary(x) => {
                let (left, operator, right) = x.destroy();
                match operator {
                    token::Kind::And => Self::new_binary(
                        left.pre_distribute()?,
                        token::Kind::And,
                        right.pre_distribute()?,
                    ),
                    token::Kind::Or => Self::new_binary(
                        left.pre_distribute()?,
                        token::Kind::Or,
                        right.pre_distribute()?,
                    ),
                    token::Kind::Implies => Self::new_binary(
                        left.negate_and_pre_distribute()?,
                        token::Kind::Or,
                        right.pre_distribute()?,
                    ),
                    token::Kind::Equiv => Self::new_binary(
                        Self::new_binary(
                            left.clone().pre_distribute()?,
                            token::Kind::And,
                            right.clone().pre_distribute()?,
                        ),
                        token::Kind::Or,
                        Self::new_binary(
                            left.negate_and_pre_distribute()?,
                            token::Kind::And,
                            right.negate_and_pre_distribute()?,
                        ),
                    ),
                    _ => {
                        return Err(InternalErrorTok::new(
                            operator,
                            "not a valid binary operator".to_string(),
                        ))
                    }
                }
            }
            Formula::Leaf(_) => self,
            Formula::Eof => {
                return Err(InternalErrorTok::new(
                    token::Kind::Eof,
                    "should not be in ast".to_string(),
                ))
            }
        })
    }

    // TODO: find a better name for this function
    pub fn negate_and_pre_distribute(self) -> Res<Self> {
        Ok(match self {
            Formula::Unary(x) => {
                let (operator, right) = x.destroy();
                match operator {
                    token::Kind::Not => right,
                    _ => {
                        return Err(InternalErrorTok::new(
                            token::Kind::Eof,
                            "not an unary opertor".to_string(),
                        ))
                    }
                }
            }
            Formula::Binary(x) => {
                let (left, operator, right) = x.destroy();
                match operator {
                    token::Kind::And => Self::new_binary(
                        left.negate_and_pre_distribute()?,
                        token::Kind::Or,
                        right.negate_and_pre_distribute()?,
                    ),
                    token::Kind::Or => Self::new_binary(
                        left.negate_and_pre_distribute()?,
                        token::Kind::And,
                        right.negate_and_pre_distribute()?,
                    ),
                    token::Kind::Implies => Self::new_binary(
                        left.pre_distribute()?,
                        token::Kind::And,
                        right.negate_and_pre_distribute()?,
                    ),
                    token::Kind::Equiv => Self::new_binary(
                        left.pre_distribute()?,
                        token::Kind::Equiv,
                        right.negate_and_pre_distribute()?,
                    ),
                    _ => {
                        return Err(InternalErrorTok::new(
                            token::Kind::Eof,
                            "not a binnary opertor".to_string(),
                        ))
                    }
                }
            }
            Formula::Leaf(_) => Self::new_unary(token::Kind::Not, self),
            Formula::Eof => {
                return Err(InternalErrorTok::new(
                    token::Kind::Eof,
                    "should not be in ast".to_string(),
                ))
            }
        })
    }

    pub fn distribute(self) -> Res<Self> {
        let formula = self.pre_distribute()?;
        Ok(match formula {
            Formula::Unary(_) => formula, // should be only before a leaf
            Formula::Binary(x) => {
                let (left, operator, right) = x.destroy();
                let left = left.distribute()?;
                let right = right.distribute()?;
                match operator {
                    token::Kind::And => Self::new_binary(left, token::Kind::And, right),
                    token::Kind::Or => {
                        if left.is_and() {
                            Self::distribute_left(left, right)?
                        } else if right.is_and() {
                            Self::distribute_right(left, right)?
                        } else {
                            Self::new_binary(left, token::Kind::Or, right)
                        }
                    }
                    token::Kind::Implies | token::Kind::Equiv => {
                        return Err(InternalErrorTok::new(
                            operator,
                            "Call simplify before".to_string(),
                        ))
                    }
                    _ => {
                        return Err(InternalErrorTok::new(
                            operator,
                            "not a valid binary operator".to_string(),
                        ))
                    }
                }
            }
            Formula::Leaf(_) => formula,
            Formula::Eof => {
                return Err(InternalErrorTok::new(
                    token::Kind::Eof,
                    "should not be in ast".to_string(),
                ))
            }
        })
    }

    /// assert left.is_and()
    // distribute recursively
    //     |              &
    //   &   c         |     |
    //  a b           a c   b c
    fn distribute_left(left: Formula, right: Formula) -> Res<Self> {
        Ok(if let Self::Binary(l) = left {
            if l.operator == token::Kind::And {
                let (a, _, b) = l.destroy();
                let c = right;
                Self::new_binary(
                    Self::new_binary(a, token::Kind::Or, c.clone()).distribute()?,
                    token::Kind::And,
                    Self::new_binary(b, token::Kind::Or, c).distribute()?,
                )
            } else {
                return Err(InternalError::new("assert left.is_and()".to_string()));
            }
        } else {
            return Err(InternalError::new("assert left.is_and()".to_string()));
        })
    }

    /// assert right.is_and()
    /// distribute recursively
    ///     |              &
    ///  c     &        |      |
    ///       a b      c a    c b
    fn distribute_right(left: Self, right: Self) -> Res<Self> {
        Ok(if let Self::Binary(r) = right {
            if r.operator == token::Kind::And {
                let (a, _, b) = r.destroy();
                let c = left;
                Self::new_binary(
                    Self::new_binary(c.clone(), token::Kind::Or, a).distribute()?,
                    token::Kind::And,
                    Self::new_binary(c, token::Kind::Or, b).distribute()?,
                )
            } else {
                return Err(InternalError::new("assert right.is_and()".to_string()));
            }
        } else {
            return Err(InternalError::new("assert right.is_and()".to_string()));
        })
    }
}

#[derive(Debug)]
pub struct InternalErrorTok {
    kind: token::Kind,
    message: String,
}
#[derive(Debug)]
pub struct InternalError {
    message: String,
}
impl Error for InternalErrorTok {}
impl Error for InternalError {}

impl InternalError {
    pub fn new(message: String) -> Box<Self> {
        Box::new(Self { message })
    }
}
impl InternalErrorTok {
    pub fn new(tok: token::Kind, message: String) -> Box<Self> {
        Box::new(Self { kind: tok, message })
    }
}
impl fmt::Display for InternalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Interanl error: {}", self.message)
    }
}
impl fmt::Display for InternalErrorTok {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Interanl error: got=`{:?}` : {}",
            self.kind, self.message
        )
    }
}

#[cfg(test)]
mod test {
    use crate::{lexer, parser::Parser, token};

    fn compare_pre_distribute(pars: &mut Parser, expected: &[&str]) {
        for &exp in expected {
            // i suppose that the parser tests pass
            let l = match pars.parse_formula().unwrap().pre_distribute() {
                Ok(s) => format!("{s}"),
                Err(s) => format!("{s}"),
            };
            if exp != l {
                panic!("expected=`{exp}`\ngot     =`{l}`")
            }
        }
    }

    #[test]
    fn test_pre_distribute() {
        let buffer = "
!x;
x & y;
x | y;
x => y;
x <=> y;
!x => !y;
x <=> y => z;
x | y => z;
";
        // i suppose that the lexer tests pass
        let mut lex_test = lexer::Lexer::new();
        lex_test.load_bytes(buffer.to_string());
        let mut tokens = Vec::new();
        while let Ok(t) = lex_test.next_tok() {
            if t.kind() == token::Kind::Eof {
                break;
            }
            tokens.push(Some(t));
        }

        let expected: &[&str] = &[
            "(!x)",
            "(x & y)",
            "(x | y)",
            "((!x) | y)",
            "((x & y) | ((!x) & (!y)))",
            "(x | (!y))",
            "((x & ((!y) | z)) | ((!x) & (y & (!z))))",
            "(((!x) & (!y)) | z)",
        ];
        let mut lex = lexer::Lexer::new();
        lex.load_bytes(buffer.to_string());
        let mut parser = Parser::new(lex).unwrap();
        compare_pre_distribute(&mut parser, expected);
    }

    fn compare_distribute(pars: &mut Parser, expected: &[&str]) {
        for &exp in expected {
            // i suppose that the parser tests pass
            let l = match pars.parse_formula().unwrap().distribute() {
                Ok(s) => format!("{s}"),
                Err(s) => format!("{s}"),
            };
            if exp != l {
                panic!("expected=`{exp}`\ngot     =`{l}`")
            }
        }
    }

    #[test]
    fn test_distribute() {
        let buffer = "
x <=> y;
a | (b & c & (d | e | (f & g)));
a & (b | c | (d & e & (f | g)));
";
        // i suppose that the lexer tests pass
        let mut lex_test = lexer::Lexer::new();
        lex_test.load_bytes(buffer.to_string());
        let mut tokens = Vec::new();
        while let Ok(t) = lex_test.next_tok() {
            if t.kind() == token::Kind::Eof {
                break;
            }
            tokens.push(Some(t));
        }

        let expected: &[&str] = &[
            "(((x | (!x)) & (x | (!y))) & ((y | (!x)) & (y | (!y))))",
            "(((a | b) & (a | c)) & ((a | ((d | e) | f)) & (a | ((d | e) | g))))",
            "(a & ((((b | c) | d) & ((b | c) | e)) & ((b | c) | (f | g))))",
        ];
        let mut lex = lexer::Lexer::new();
        lex.load_bytes(buffer.to_string());
        let mut parser = Parser::new(lex).unwrap();
        compare_distribute(&mut parser, expected);
    }
}
