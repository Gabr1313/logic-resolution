use core::fmt;
use std::error::Error;

use crate::Res;
use crate::{ast, lexer, token};

#[derive(Debug)]
pub struct Parser {
    lex: lexer::Lexer,
    curr_tok: Option<token::Token>,
    peek_tok: Option<token::Token>,
}

impl Parser {
    pub fn new(lex: lexer::Lexer) -> Res<Self> {
        let mut p = Parser {
            lex,
            curr_tok: None,
            peek_tok: None,
        };
        p.curr_tok = p.peek_tok.take();
        p.peek_tok = Some(p.lex.next_tok()?);
        p.curr_tok = p.peek_tok.take();
        p.peek_tok = Some(p.lex.next_tok()?);
        Ok(p)
    }

    pub fn load_bytes(&mut self, buffer: String) -> Res<()> {
        self.lex.load_bytes(buffer);
        self.curr_tok = self.peek_tok.take();
        self.peek_tok = Some(self.lex.next_tok()?);
        self.curr_tok = self.peek_tok.take();
        self.peek_tok = Some(self.lex.next_tok()?);
        Ok(())
    }

    pub fn curr_tok(&self) -> &token::Token {
        // should never panic because None is only the initial value
        self.curr_tok.as_ref().unwrap()
    }

    /// returns the previous token
    fn skip_tok(&mut self) -> Res<token::Token> {
        // should never panic because None is only the initial value
        let tok = self.curr_tok.take().unwrap();
        self.curr_tok = self.peek_tok.take();
        self.peek_tok = Some(self.lex.next_tok()?);
        Ok(tok)
    }

    pub fn parse_statement(&mut self) -> Res<ast::Formula> {
        self.recursive_pratt(0)
    }

    fn recursive_pratt(&mut self, precedence: usize) -> Res<ast::Formula> {
        // pre: unary
        let mut formula = match self.curr_tok().kind {
            token::Kind::Not => self.parse_unary(),
            token::Kind::ParenL => self.parse_paren(),
            token::Kind::Ident => self.parse_leaf(),
            _ => {
                return Err(ParseErr::new(
                    self.skip_tok()?,
                    "expected `identifier` or `unary operator`",
                ))
            }
        };

        while precedence < self.curr_tok().precedence() {
            match self.curr_tok().kind {
                token::Kind::And | token::Kind::Or | token::Kind::Implies | token::Kind::Equiv => {
                    formula = self.parse_binary(formula?)
                }
                token::Kind::SemiColon => {
                    self.skip_tok()?;
                    break;
                }
                _ => {
                    return Err(ParseErr::new(
                        self.skip_tok()?,
                        "expected `;` or `binary operator`",
                    ))
                }
            }
        }
        formula
    }

    fn parse_paren(&mut self) -> Res<ast::Formula> {
        let paren_l = self.skip_tok()?;
        let f = self.recursive_pratt(paren_l.precedence())?;
        if self.curr_tok().kind != token::Kind::ParenR {
            todo!("Error: expected found )")
        }
        self.skip_tok()?;
        Ok(f)
    }

    fn parse_leaf(&mut self) -> Res<ast::Formula> {
        Ok(ast::Formula::new_leaf(self.skip_tok()?))
    }

    fn parse_unary(&mut self) -> Res<ast::Formula> {
        let operator = self.skip_tok()?;
        let p = operator.precedence();
        Ok(ast::Formula::new_unary(operator, self.recursive_pratt(p)?))
    }

    fn parse_binary(&mut self, left: ast::Formula) -> Res<ast::Formula> {
        let operator = self.skip_tok()?;
        let p = operator.precedence();
        Ok(ast::Formula::new_binary(
            left,
            operator,
            self.recursive_pratt(p)?,
        ))
    }
}

#[derive(Debug)]
pub struct ParseErr<'a> {
    tok: token::Token,
    message: &'a str,
}

impl<'a> Error for ParseErr<'a> {}

impl<'a> ParseErr<'a> {
    pub fn new(tok: token::Token, message: &'a str) -> Box<Self> {
        Box::new(ParseErr { tok, message })
    }
}

impl<'a> fmt::Display for ParseErr<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Parse error [{}:{}]: got=`{}` ({:?}): {}",
            self.tok.row, self.tok.col, self.tok.literal, self.tok.kind, self.message
        )
    }
}

#[cfg(test)]
mod test {
    use super::Parser;
    use crate::lexer;
    use crate::token;

    fn compare(pars: &mut Parser, expected: &[&str]) {
        for &exp in expected {
            let l = format!(
                "{}",
                pars.parse_statement().unwrap_or_else(|e| panic!("{e}"))
            );
            if exp != l {
                panic!("expected=`{exp}`\ngot     =`{l}`")
            }
        }
    }

    #[test]
    fn test_parser() {
        let buffer = "
!x;
x => y;
x | y;
x & y;
x <=> y;
((x | y)) & z;
x <=> y => z | w & !v;
!x & y | z => w <=> v;
!x | (y | z) <=> !w => v & b;
";
        // i suppose that the lexer test passes
        let mut lex_test = lexer::Lexer::new();
        lex_test.load_bytes(buffer.to_string());
        let mut tokens = Vec::new();
        while let Ok(t) = lex_test.next_tok() {
            if t.kind == token::Kind::Eof {
                break;
            }
            tokens.push(Some(t));
        }

        let expected: &[&str] = &[
            "(!x)",
            "(x => y)",
            "(x | y)",
            "(x & y)",
            "(x <=> y)",
            "((x | y) & z)",
            "(x <=> (y => (z | (w & (!v)))))",
            "(((((!x) & y) | z) => w) <=> v)",
            "(((!x) | (y | z)) <=> ((!w) => (v & b)))",
        ];
        let mut lex = lexer::Lexer::new();
        lex.load_bytes(buffer.to_string());
        let mut parser = Parser::new(lex).unwrap();
        compare(&mut parser, expected);
    }
}
