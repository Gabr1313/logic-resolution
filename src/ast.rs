use crate::{token, Res};
use std::fmt;

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

    /// push `!` inside, and substitue `=>` and`<=>`
    pub fn simplify(self) -> Res<Self> {
        Ok(match self {
            Formula::Unary(x) => {
                let (operator, right) = x.destroy();
                match operator {
                    token::Kind::Not => right.negate_and_simplify()?,
                    _ => todo!("this is not an unary opertor"),
                }
            }
            Formula::Binary(x) => {
                let (left, operator, right) = x.destroy();
                match operator {
                    token::Kind::And => {
                        Self::new_binary(left.simplify()?, token::Kind::And, right.simplify()?)
                    }
                    token::Kind::Or => {
                        Self::new_binary(left.simplify()?, token::Kind::Or, right.simplify()?)
                    }
                    token::Kind::Implies => Self::new_binary(
                        left.negate_and_simplify()?,
                        token::Kind::Or,
                        right.simplify()?,
                    ),
                    token::Kind::Equiv => Self::new_binary(
                        Self::new_binary(
                            left.clone().simplify()?,
                            token::Kind::And,
                            right.clone().simplify()?,
                        ),
                        token::Kind::Or,
                        Self::new_binary(
                            left.negate_and_simplify()?,
                            token::Kind::And,
                            right.negate_and_simplify()?,
                        ),
                    ),
                    _ => todo!("this is not a binary opertor"),
                }
            }
            Formula::Leaf(_) => self,
            Formula::Eof => todo!("should not be in ast"),
        })
    }

    pub fn negate_and_simplify(self) -> Res<Self> {
        Ok(match self {
            Formula::Unary(x) => {
                let (operator, right) = x.destroy();
                match operator {
                    token::Kind::Not => right,
                    _ => todo!("this is not an unary opertor"),
                }
            }
            Formula::Binary(x) => {
                let (left, operator, right) = x.destroy();
                match operator {
                    token::Kind::And => Self::new_binary(
                        left.negate_and_simplify()?,
                        token::Kind::Or,
                        right.negate_and_simplify()?,
                    ),
                    token::Kind::Or => Self::new_binary(
                        left.negate_and_simplify()?,
                        token::Kind::And,
                        right.negate_and_simplify()?,
                    ),
                    token::Kind::Implies => Self::new_binary(
                        left.simplify()?,
                        token::Kind::And,
                        right.negate_and_simplify()?,
                    ),
                    token::Kind::Equiv => Self::new_binary(
                        left.simplify()?,
                        token::Kind::Equiv,
                        right.negate_and_simplify()?,
                    ),
                    _ => todo!("this is not a binary opertor"),
                }
            }
            Formula::Leaf(_) => Self::new_unary(token::Kind::Not, self),
            Formula::Eof => todo!("should not be in ast"),
        })
    }

    pub fn distribute(self) -> Res<Self> {
        Ok(match self {
            Formula::Unary(_) => self, // should be only before a leaf
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
                    token::Kind::Implies => todo!("should never happen"),
                    token::Kind::Equiv => todo!("should never happen"),
                    _ => todo!("this is not an binary opertor"),
                }
            }
            Formula::Leaf(_) => self,
            Formula::Eof => todo!("should not be in ast"),
        })
    }

    /// distribute recursively
    ///     |              &
    ///   &   c         |     |
    ///  a b           a c   b c
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
                todo!("impossible");
            }
        } else {
            todo!("impossible");
        })
    }

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
                todo!("impossible");
            }
        } else {
            todo!("impossible");
        })
    }
}

#[cfg(test)]
mod test {
    use crate::{lexer, parser::Parser, token};

    fn compare_simplify(pars: &mut Parser, expected: &[&str]) {
        for &exp in expected {
            // i suppose that the parser tests pass
            let l = match pars.parse_formula().unwrap().simplify() {
                Ok(s) => format!("{s}"),
                Err(s) => format!("{s}"),
            };
            if exp != l {
                panic!("expected=`{exp}`\ngot     =`{l}`")
            }
        }
    }

    #[test]
    fn test_simplify() {
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
        compare_simplify(&mut parser, expected);
    }

    fn compare_distribute(pars: &mut Parser, expected: &[&str]) {
        for &exp in expected {
            // i suppose that the parser and the simplify tests pass
            let l = match pars
                .parse_formula()
                .unwrap()
                .simplify()
                .unwrap()
                .distribute()
            {
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
