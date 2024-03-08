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
    pub fn operator(&self) -> token::Kind {
        self.operator
    }
    pub fn right(&self) -> &Formula {
        &self.right
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
    pub fn string(&self) -> String {
        // TODO: no close
        self.ident.to_string()
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
    pub fn new_leaf(ident: String) -> Formula {
        Formula::Leaf(Leaf { ident })
    }
    pub fn new_binary(left: Formula, operator: token::Kind, right: Formula) -> Formula {
        Formula::Binary(Binary {
            left: Box::new(left),
            operator,
            right: Box::new(right),
        })
    }
    pub fn new_unary(operator: token::Kind, right: Formula) -> Formula {
        Formula::Unary(Unary {
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
    /// push `!` inside (simplifying if repeated), and substitue `=>` and`<=>`
    fn digest(self) -> Res<Formula> {
        Ok(match self {
            Formula::Unary(x) => {
                let (operator, right) = x.destroy();
                match operator {
                    token::Kind::Not => right.negate_digest()?,
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
                    token::Kind::And => Formula::new_binary(
                        left.digest()?,
                        token::Kind::And,
                        right.digest()?,
                    ),
                    token::Kind::Or => Formula::new_binary(
                        left.digest()?,
                        token::Kind::Or,
                        right.digest()?,
                    ),
                    token::Kind::Implies => Formula::new_binary(
                        left.negate_digest()?,
                        token::Kind::Or,
                        right.digest()?,
                    ),
                    token::Kind::Equiv => Formula::new_binary(
                        Formula::new_binary(
                            left.clone().digest()?,
                            token::Kind::And,
                            right.clone().digest()?,
                        ),
                        token::Kind::Or,
                        Formula::new_binary(
                            left.negate_digest()?,
                            token::Kind::And,
                            right.negate_digest()?,
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
    pub fn negate_digest(self) -> Res<Formula> {
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
                    token::Kind::And => Formula::new_binary(
                        left.negate_digest()?,
                        token::Kind::Or,
                        right.negate_digest()?,
                    ),
                    token::Kind::Or => Formula::new_binary(
                        left.negate_digest()?,
                        token::Kind::And,
                        right.negate_digest()?,
                    ),
                    token::Kind::Implies => Formula::new_binary(
                        left.digest()?,
                        token::Kind::And,
                        right.negate_digest()?,
                    ),
                    token::Kind::Equiv => Formula::new_binary(
                        left.digest()?,
                        token::Kind::Equiv,
                        right.negate_digest()?,
                    ),
                    _ => {
                        return Err(InternalErrorTok::new(
                            token::Kind::Eof,
                            "not a binnary opertor".to_string(),
                        ))
                    }
                }
            }
            Formula::Leaf(_) => Formula::new_unary(token::Kind::Not, self),
            Formula::Eof => {
                return Err(InternalErrorTok::new(
                    token::Kind::Eof,
                    "should not be in ast".to_string(),
                ))
            }
        })
    }

    pub fn distribute(self) -> Res<Formula> {
        let formula = self.digest()?;
        formula.distribute_recurse()
    }

    fn distribute_recurse(self) -> Res<Formula> {
        Ok(match self {
            Formula::Unary(_) => self, // should be only before a leaf
            Formula::Binary(x) => {
                let (left, operator, right) = x.destroy();
                let left = left.distribute_recurse()?;
                let right = right.distribute_recurse()?;
                match operator {
                    token::Kind::And => Formula::new_binary(left, token::Kind::And, right),
                    token::Kind::Or => {
                        if left.is_and() {
                            Formula::distribute_left(left, right)?
                        } else if right.is_and() {
                            Formula::distribute_right(left, right)?
                        } else {
                            Formula::new_binary(left, token::Kind::Or, right)
                        }
                    }
                    token::Kind::Implies | token::Kind::Equiv => {
                        return Err(InternalErrorTok::new(
                            operator,
                            "call self.digest() before".to_string(),
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
            Formula::Leaf(_) => self,
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
    fn distribute_left(left: Formula, right: Formula) -> Res<Formula> {
        Ok(if let Formula::Binary(l) = left {
            if l.operator == token::Kind::And {
                let (a, _, b) = l.destroy();
                let c = right;
                Formula::new_binary(
                    Formula::new_binary(a, token::Kind::Or, c.clone()).distribute_recurse()?,
                    token::Kind::And,
                    Formula::new_binary(b, token::Kind::Or, c).distribute_recurse()?,
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
    fn distribute_right(left: Formula, right: Formula) -> Res<Formula> {
        Ok(if let Formula::Binary(r) = right {
            if r.operator == token::Kind::And {
                let (a, _, b) = r.destroy();
                let c = left;
                Formula::new_binary(
                    Formula::new_binary(c.clone(), token::Kind::Or, a).distribute_recurse()?,
                    token::Kind::And,
                    Formula::new_binary(c, token::Kind::Or, b).distribute_recurse()?,
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
    pub fn new(message: String) -> Box<InternalError> {
        Box::new(InternalError { message })
    }
}
impl InternalErrorTok {
    pub fn new(tok: token::Kind, message: String) -> Box<InternalErrorTok> {
        Box::new(InternalErrorTok { kind: tok, message })
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

    fn compare_digest(pars: &mut Parser, expected: &[&str]) {
        for &exp in expected {
            // i suppose that the parser tests pass
            let l = match pars.parse_formula().unwrap().digest() {
                Ok(s) => format!("{s}"),
                Err(s) => format!("{s}"),
            };
            if exp != l {
                panic!("expected=`{exp}`\ngot     =`{l}`")
            }
        }
    }

    #[test]
    fn test_digest() {
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
        compare_digest(&mut parser, expected);
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
